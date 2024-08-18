#![no_std]
#![no_main]

extern crate alloc;

use core::{arch::asm, panic::PanicInfo};

use log::{error, info};
use uefi::{
    entry,
    Handle,
    Status, table::{Boot, boot::MemoryType, SystemTable},
};

use core_util::{BootInfo, memory::PAGE_SIZE};

mod file;
mod graphics;
mod memory;

const KERNEL_FILE_NAME: &str = "kernel.elf";
const KERNEL_STACK_SIZE: usize = 1024 * 1024; // 1MiB
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
    let framebuffer_address = framebuffer_metadata.base;
    let framebuffer_page_count = (framebuffer_metadata.size + PAGE_SIZE - 1) / PAGE_SIZE;

    // allocate kernel stack
    let (kernel_stack_address, kernel_stack_page_count) =
        memory::allocate_stack(boot_services).unwrap();

    // allocate boot info
    let boot_info_address = memory::allocate_boot_info(boot_services).unwrap();

    // set up address space
    let (pml4, rsp, virtual_boot_info_address) = memory::set_up_address_space(
        boot_services,
        memory::KernelInfo {
            kernel_code_address,
            kernel_code_page_count,
            kernel_stack_address,
            kernel_stack_page_count,
            framebuffer_address,
            framebuffer_page_count,
            boot_info_address,
        },
    )
    .unwrap();

    let boot_info = unsafe { &mut *(boot_info_address as *mut BootInfo) };
    boot_info.frame_buffer_metadata = framebuffer_metadata;
    // exit boot services
    let (_runtime, _mmap) = unsafe { system_table.exit_boot_services(MemoryType::LOADER_DATA) };

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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: \n{:#?}", info);
    loop {}
}
