use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
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
    },
    PhysicalAddress, pmm::{BitMapAllocator, PageFrameAllocatorError},
};

use crate::{CoreMemoryDescriptor, CoreMemoryMap, KERNEL_STACK_SIZE};

#[derive(Clone, Debug)]
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
        .allocate_pages(AnyPages, MemoryType::LOADER_DATA, num_pages)
        .map_err(|_| {
            format!(
                "Could not allocate {} pages for the kernel stack.",
                num_pages
            )
        })?;
    Ok((start_addr, num_pages))
}
/// Allocate a single page to store the boot information in. As well as uefi memory map descriptors
pub(super) fn allocate_boot_info(
    bt: &BootServices,
) -> Result<(PhysicalAddress, Vec<CoreMemoryDescriptor>), String> {
    let boot_info_addr = bt
        .allocate_pages(AnyPages, MemoryType::LOADER_DATA, 1)
        .map_err(|_| "Could not allocate page for kernel boot information.".to_string())?;

    // get uefi mmap meta data to allocate enough later for custom memory map in `drop_boot_services`
    let uefi_memory_map_meta = bt
        .memory_map(MemoryType::LOADER_DATA)
        .map_err(|error| format!("Could not get uefi memory map: {error}"))?
        .as_raw()
        .1;

    // allocate enough memory for the map. Add additional padding in case map size changes
    let sufficient_memory_map_size = uefi_memory_map_meta.entry_count() + 8;

    // allocate descriptors in memory
    let descriptors = Vec::with_capacity(sufficient_memory_map_size);

    Ok((boot_info_addr, descriptors))
}
/// Sets up paging that includes mappings for higher half kernel and higher half stack. Returns address pointing to page table manager, stack pointer and pointer to boot info.
pub(super) fn set_up_address_space(
    memory_map: &CoreMemoryMap,
    kernel_info: KernelInfo,
) -> Result<(PhysicalAddress, VirtualAddress, VirtualAddress), PageFrameAllocatorError> {
    let KernelInfo {
        kernel_code_address,
        kernel_code_page_count,
        kernel_stack_address,
        kernel_stack_page_count,
        framebuffer_address,
        framebuffer_page_count,
        boot_info_address,
    } = kernel_info;

    // set up physical memory manager
    let mut pmm = BitMapAllocator::try_new(*memory_map)?;

    let pml4_addr = pmm.request_page()?;
    assert_eq!(
        (pml4_addr as usize) % align_of::<PageTable>(),
        0,
        "pml4 pointer is not aligned"
    );

    let pml4_table = pml4_addr as *mut PageTable;

    // zero out new table
    unsafe { ptr::write_bytes(pml4_table, 0, 1) };

    let mut manager: PageTableManager<BitMapAllocator, PageFrameAllocatorError> =
        PageTableManager::new(pml4_table, pmm);
    let first_addr = memory_map.first_addr;
    let last_addr = memory_map.last_addr;
    let page_count = ((last_addr - first_addr) as usize + PAGE_SIZE - 1) / PAGE_SIZE;
    // identity map entire available physical address space
    for page in 0..page_count {
        let physical_address = (PAGE_SIZE * page) as u64 + first_addr;
        manager.map_memory(
            physical_address,
            physical_address,
            PageEntryFlags::default(),
        )?;
    }

    // map higher half kernel virtual addresses to physical kernel addresses
    for page in 0..kernel_code_page_count {
        let physical_address = ((PAGE_SIZE * page) as u64) + kernel_code_address;
        let virtual_address = KERNEL_MAPPING_OFFSET + physical_address;
        manager.map_memory(virtual_address, physical_address, PageEntryFlags::default())?;
    }

    // map boot info page to higher half directly after kernel
    let virtual_boot_info_address =
        KERNEL_MAPPING_OFFSET + (PAGE_SIZE * kernel_code_page_count) as u64;
    manager.map_memory(
        virtual_boot_info_address,
        boot_info_address,
        PageEntryFlags::default(),
    )?;

    // map stack to higher half offset
    for page in 0..kernel_stack_page_count {
        let physical_address = (PAGE_SIZE * page) as u64 + kernel_stack_address;
        let virtual_address = (PAGE_SIZE * page) as u64 + KERNEL_STACK_MAPPING_OFFSET;
        manager.map_memory(virtual_address, physical_address, PageEntryFlags::default())?;
    }

    // identity map framebuffer
    for page in 0..framebuffer_page_count {
        let physical_address = (PAGE_SIZE * page) as u64 + framebuffer_address;
        manager.map_memory(
            physical_address,
            physical_address,
            PageEntryFlags::default(),
        )?;
    }

    Ok((
        pml4_addr,
        KERNEL_STACK_MAPPING_OFFSET + KERNEL_STACK_SIZE as u64,
        virtual_boot_info_address,
    ))
}
