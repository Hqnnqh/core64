use alloc::{
    format,
    string::{String, ToString},
};
use core::ptr;

use uefi::{
    data_types::VirtualAddress,
    prelude::BootServices,
    table::boot::{AllocateType::AnyPages, MemoryType},
};

use core_util::memory::{
    PAGE_SIZE,
    paging::{
        KERNEL_MAPPING_OFFSET,
        KERNEL_STACK_MAPPING_OFFSET, manager::{PageFrameAllocator, PageTableManager}, PageEntryFlags, PageTable,
    }, PhysicalAddress,
};

use crate::KERNEL_STACK_SIZE;

pub(super) const KERNEL_CODE: MemoryType = MemoryType::custom(0x80000000);
pub(super) const KERNEL_STACK: MemoryType = MemoryType::custom(0x80000001);
pub(super) const KERNEL_DATA: MemoryType = MemoryType::custom(0x80000002);

pub(super) const LOADER_PAGING: MemoryType = MemoryType::custom(0x80000003);

pub(super) struct KernelInfo {
    pub(super) kernel_code_address: PhysicalAddress,
    pub(super) kernel_code_page_count: usize,
    pub(super) kernel_stack_address: PhysicalAddress,
    pub(super) kernel_stack_page_count: usize,
    pub(super) framebuffer_address: PhysicalAddress,
    pub(super) framebuffer_page_count: usize,
    pub(super) boot_info_address: PhysicalAddress,
}

/// Allocate pages for kernel stack. Returns physical address of allocated stack and amount of pages allocated.
pub(super) fn allocate_stack(bt: &BootServices) -> Result<(PhysicalAddress, usize), String> {
    let num_pages = (KERNEL_STACK_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;
    let start_addr = bt
        .allocate_pages(AnyPages, KERNEL_STACK, num_pages)
        .map_err(|_| {
            format!(
                "Could not allocate {} pages for the kernel stack.",
                num_pages
            )
        })?;
    Ok((start_addr, num_pages))
}
pub(super) fn allocate_boot_info(bt: &BootServices) -> Result<PhysicalAddress, String> {
    // allocate 1 page for boot info struct
    bt.allocate_pages(AnyPages, KERNEL_DATA, 1)
        .map_err(|_| "Could not allocate page for kernel boot information.".to_string())
}
/// Sets up paging that includes mappings for higher half kernel and higher half stack. Returns address pointing to page table manager, stack pointer and pointer to boot info.
pub(super) fn set_up_address_space(
    boot_services: &BootServices,
    kernel_info: KernelInfo,
) -> Result<(PhysicalAddress, VirtualAddress, VirtualAddress), String> {
    let KernelInfo {
        kernel_code_address,
        kernel_code_page_count,
        kernel_stack_address,
        kernel_stack_page_count,
        framebuffer_address,
        framebuffer_page_count,
        boot_info_address,
    } = kernel_info;

    let pml4_addr = boot_services
        .allocate_pages(AnyPages, LOADER_PAGING, 1)
        .map_err(|_| "Could not allocate new page table".to_string())?;

    assert_eq!(
        (pml4_addr as usize) % align_of::<PageTable>(),
        0,
        "pml4 pointer is not aligned"
    );

    let pml4_table = pml4_addr as *mut PageTable;

    // zero out new table
    unsafe { ptr::write_bytes(pml4_table, 0, 1) };

    let page_frame_allocator = BootServiceWrapper(boot_services);

    let mut manager: PageTableManager<BootServiceWrapper, String> =
        PageTableManager::new(pml4_table, page_frame_allocator);

    let mmap = boot_services
        .memory_map(MemoryType::LOADER_DATA)
        .map_err(|_| "Could not get memory map.".to_string())?;

    let first_addr = mmap
        .entries()
        .filter(|desc| {
            matches!(
                desc.ty,
                MemoryType::CONVENTIONAL
                    | MemoryType::BOOT_SERVICES_DATA
                    | MemoryType::BOOT_SERVICES_CODE
            ) && desc.phys_start > 0x0
        }) // skip areas like 0x0
        .map(|desc| desc.phys_start)
        .min()
        .ok_or("Memory map is empty".to_string())?;
    let last_addr = mmap
        .entries()
        .filter(|desc| {
            matches!(
                desc.ty,
                MemoryType::CONVENTIONAL
                    | MemoryType::BOOT_SERVICES_DATA
                    | MemoryType::BOOT_SERVICES_CODE
            )
        })
        .map(|desc| desc.phys_start + PAGE_SIZE as u64 * desc.page_count)
        .max()
        .ok_or("Memory map is empty".to_string())?;
    let page_count = ((last_addr - first_addr) as usize + PAGE_SIZE - 1) / PAGE_SIZE;
    for page in 0..page_count {
        let physical_address = (PAGE_SIZE * page) as u64 + first_addr;
        manager
            .map_memory(
                physical_address,
                physical_address,
                PageEntryFlags::default(),
            )
            .map_err(|_| {
                format!(
                    "Could not identity map physical address: {:#x}.",
                    physical_address
                )
            })?;
    }

    // map higher half kernel virtual addresses to physical kernel addresses
    for page in 0..kernel_code_page_count {
        let physical_address = ((PAGE_SIZE * page) as u64) + kernel_code_address;
        let virtual_address = KERNEL_MAPPING_OFFSET + physical_address;
        manager
            .map_memory(virtual_address, physical_address, PageEntryFlags::default())
            .map_err(|_| {
                format!(
                    "Could not map kernel physical address: {:#x} to higher half address: {:#x}.",
                    physical_address, virtual_address
                )
            })?;
    }

    // map boot info page to higher half directly after kernel
    let virtual_boot_info_address =
        KERNEL_MAPPING_OFFSET + (PAGE_SIZE * kernel_code_page_count) as u64;
    manager
        .map_memory(
            virtual_boot_info_address,
            boot_info_address,
            PageEntryFlags::default(),
        )
        .map_err(|_| {
            format!(
                "Could not map boot info physical address: {:#x} to higher half address: {:#x}.",
                boot_info_address, virtual_boot_info_address
            )
        })?;

    // map stack to higher half offset
    for page in 0..kernel_stack_page_count {
        let physical_address = (PAGE_SIZE * page) as u64 + kernel_stack_address;
        let virtual_address = (PAGE_SIZE * page) as u64 + KERNEL_STACK_MAPPING_OFFSET;
        manager
            .map_memory(virtual_address, physical_address, PageEntryFlags::default())
            .map_err(|_| {
                format!(
                    "Could not map kernel stack physical address: {:#x} to higher half address: {:#x}.",
                    physical_address, virtual_address
                )
            })?;
    }

    // identity map framebuffer
    for page in 0..framebuffer_page_count {
        let physical_address = (PAGE_SIZE * page) as u64 + framebuffer_address;
        manager
            .map_memory(
                physical_address,
                physical_address,
                PageEntryFlags::default(),
            )
            .map_err(|_| {
                format!(
                    "Could not identity map framebuffer physical address: {:#x}",
                    physical_address
                )
            })?;
    }

    Ok((
        pml4_addr,
        KERNEL_STACK_MAPPING_OFFSET + KERNEL_STACK_SIZE as u64,
        virtual_boot_info_address
    ))
}

/// Wrapper for BootServices that allows PageFrameAllocator implementation
struct BootServiceWrapper<'a>(&'a BootServices);

impl<'a> PageFrameAllocator<'a, String> for BootServiceWrapper<'a> {
    fn request_page(&mut self) -> Result<PhysicalAddress, String> {
        self.0
            .allocate_pages(AnyPages, LOADER_PAGING, 1)
            .map_err(|_| "Could not allocate page for page table manager.".to_string())
    }
}
