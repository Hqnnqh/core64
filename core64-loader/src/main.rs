#![no_std]
#![no_main]
#![feature(vec_into_raw_parts)]

extern crate alloc;

use alloc::vec::Vec;
use core::{arch::asm, panic::PanicInfo};

use log::{error, info};
use uefi::{
    entry,
    Handle,
    Status, table::{Boot, boot::MemoryType, Runtime, SystemTable},
};

use core64_util::{BootInfo, memory::PAGE_SIZE};

use crate::memory::KernelInfo;

mod file;
mod graphics;
mod memory;

const KERNEL_FILE_NAME: &str = "kernel.elf";
const KERNEL_STACK_SIZE: usize = 1024 * 1024; // 1MiB

type CoreMemoryMap = core64_util::memory::MemoryMap;
type CoreMemoryDescriptor = core64_util::memory::MemoryDescriptor;
type CoreMemoryType = core64_util::memory::MemoryType;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();

    info!("Hello Bootloader :) => Booting kernel...");

    // load file data
    let kernel_file_data =
        file::get_file_data(image_handle, boot_services, KERNEL_FILE_NAME).unwrap();

    // parse elf
    let (kernel_entry_address, kernel_code_address, kernel_code_page_count) =
        file::parse_elf(kernel_file_data, boot_services).unwrap();

    // initialize framebuffer
    let framebuffer_metadata = graphics::initialize_framebuffer(boot_services).unwrap();

    // allocate kernel stack
    let (kernel_stack_address, kernel_stack_page_count) =
        memory::allocate_stack(boot_services).unwrap();

    // allocate boot info
    let (boot_info_address, mmap_descriptors) = memory::allocate_boot_info(boot_services).unwrap();

    let kernel_info = KernelInfo {
        kernel_code_address,
        kernel_code_page_count,
        kernel_stack_address,
        kernel_stack_page_count,
        boot_info_address,
    };
    // exit boot services
    let (_runtime, memory_map) = drop_boot_services(system_table, mmap_descriptors, &kernel_info);

    // set up address space
    let (pml4, rsp, virtual_boot_info_address) =
        memory::set_up_address_space(&memory_map, kernel_info).unwrap();

    let boot_info = unsafe { &mut *(boot_info_address as *mut BootInfo) };
    boot_info.frame_buffer_metadata = framebuffer_metadata;

    unsafe {
        asm!(
            // boot info address
            "mov rdi, {0}",
            // switch to custom paging
            "mov cr3, {2}",
            // set stack pointer to kernel stack top
            "mov rsp, {1}",
            // jump to kernel entry
            "jmp {3}",
            in(reg) virtual_boot_info_address,
            in(reg) rsp,
            in(reg) pml4,
            in(reg) kernel_entry_address
        );
    }
    // should never happen
    Status::ABORTED
}

/// Drops boot services and returns converted memory map and runtime system table
fn drop_boot_services(
    system_table: SystemTable<Boot>,
    mut descriptors: Vec<CoreMemoryDescriptor>,
    kernel_info: &KernelInfo,
) -> (SystemTable<Runtime>, CoreMemoryMap) {
    // drop boot services
    let (runtime, uefi_mmap) = unsafe { system_table.exit_boot_services(MemoryType::LOADER_DATA) };

    let mut first_addr = u64::MAX;
    let mut first_available_addr = u64::MAX;
    let mut last_addr = u64::MIN;
    let mut last_available_addr = u64::MIN;
    let desc_start_addr = descriptors.as_ptr() as u64;
    let desc_end_addr =
        desc_start_addr + (descriptors.capacity() * size_of::<CoreMemoryDescriptor>()) as u64;
    // collect available memory descriptors (convert uefi mmap to core64 mmap)
    uefi_mmap.entries().for_each(|descriptor| {
        let phys_end = descriptor.phys_start + descriptor.page_count * PAGE_SIZE as u64;

        if descriptor.phys_start < first_addr {
            first_addr = descriptor.phys_start;
        }

        if descriptor.phys_start != 0x0
            && matches!(
                descriptor.ty,
                MemoryType::CONVENTIONAL
                    | MemoryType::BOOT_SERVICES_CODE
                    | MemoryType::BOOT_SERVICES_DATA
            )
        {
            if descriptor.phys_start < first_available_addr {
                first_available_addr = descriptor.phys_start;
            }
            if phys_end > last_available_addr {
                last_available_addr = phys_end;
            }
        }

        if phys_end > last_addr {
            last_addr = phys_end;
        }

        let r#type = if descriptor.phys_start < 0x1000 {
            CoreMemoryType::Reserved
        }
        // mark mmap data as kernel data and boot info struct
        else if (descriptor.phys_start <= desc_start_addr && phys_end >= desc_end_addr)
            || descriptor.phys_start <= kernel_info.boot_info_address
                && phys_end >= kernel_info.boot_info_address + PAGE_SIZE as u64
        {
            CoreMemoryType::KernelData
        }
        // mark kernel file as kernel code
        else if descriptor.phys_start <= kernel_info.kernel_code_address
            && phys_end
                >= kernel_info.kernel_code_address
                    + (kernel_info.kernel_code_page_count * PAGE_SIZE) as u64
        {
            CoreMemoryType::KernelCode
        }
        // mark stack as kernel stack
        else if descriptor.phys_start <= kernel_info.kernel_stack_address
            && phys_end
                >= kernel_info.kernel_stack_address
                    + (kernel_info.kernel_stack_page_count * PAGE_SIZE) as u64
        {
            CoreMemoryType::KernelStack
        } else {
            // Determine the core memory type based on the UEFI memory type
            match descriptor.ty {
                MemoryType::CONVENTIONAL
                | MemoryType::BOOT_SERVICES_DATA
                | MemoryType::BOOT_SERVICES_CODE => CoreMemoryType::Available,
                _ => CoreMemoryType::Reserved,
            }
        };

        descriptors.push(CoreMemoryDescriptor {
            phys_start: descriptor.phys_start,
            phys_end,
            num_pages: descriptor.page_count,
            r#type,
        });
    });

    let (ptr, len, _cap) = descriptors.into_raw_parts();
    (
        runtime,
        CoreMemoryMap {
            descriptors: ptr as *mut CoreMemoryDescriptor,
            descriptors_len: len as u64,
            first_addr,
            first_available_addr,
            last_addr,
            last_available_addr,
        },
    )
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: \n{:#?}", info);
    loop {}
}
