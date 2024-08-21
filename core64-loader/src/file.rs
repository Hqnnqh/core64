use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::slice;

use goblin::{elf64::program_header::PT_LOAD, elf::Elf};
use uefi::{
    CString16,
    fs::FileSystem,
    Handle,
    prelude::BootServices, table::boot::{AllocateType, PAGE_SIZE},
};
use uefi::data_types::PhysicalAddress;
use uefi::table::boot::MemoryType;
use core64_util::memory::VirtualAddress;

/// Gets data of a file from filesystem
pub(super) fn get_file_data(
    image_handle: Handle,
    boot_services: &BootServices,
    filename: &str,
) -> Result<Vec<u8>, String> {
    let mut file_system = FileSystem::new(
        boot_services
            .get_image_file_system(image_handle)
            .map_err(|_| "Cannot get filesystem.".to_string())?,
    );
    file_system
        .read(
            CString16::try_from(filename)
                .map_err(|_| format!("Invalid filename: {filename}"))?
                .as_ref(),
        )
        .map_err(|_| format!("Unable to read file with name: {filename}."))
}

/// Allocates the file data in memory. Returns elf entry address, file start address and file page count
pub(super) fn parse_elf(
    data: Vec<u8>,
    boot_services: &BootServices,
) -> Result<(VirtualAddress, PhysicalAddress, usize), String> {
    let data = data.as_slice();
    let elf = Elf::parse(data).map_err(|_| "Unable to parse file to elf!".to_string())?;

    if !elf.is_64 {
        return Err("Invalid elf format.".to_string());
    }

    let mut dest_start = u64::MAX;
    let mut dest_end = 0;

    // set up range of memory needed to be allocated
    for pheader in elf.program_headers.iter() {
        // skip non-load segments (e.g.: dynamic linking info)
        if pheader.p_type != PT_LOAD {
            continue;
        }

        dest_start = dest_start.min(pheader.p_paddr);
        dest_end = dest_end.max(pheader.p_paddr + pheader.p_memsz);
    }

    let num_pages = (dest_end as usize - dest_start as usize + PAGE_SIZE - 1) / PAGE_SIZE;

    // allocate file data
    boot_services
        .allocate_pages(AllocateType::Address(dest_start), MemoryType::LOADER_DATA, num_pages)
        .map_err(|error| format!("Could not allocate pages for kernel: {}", error))?;

    // Copy program segments of kernel into memory
    for pheader in elf.program_headers.iter() {
        // skip non-load segments (e.g.: dynamic linking info) and segments of size 0 in the file
        if pheader.p_type != PT_LOAD {
            continue;
        }
        let base_address = pheader.p_paddr;
        let offset = pheader.p_offset as usize;
        let size_in_file = pheader.p_filesz as usize;
        let size_in_memory = pheader.p_memsz as usize;

        let dest = unsafe { slice::from_raw_parts_mut(base_address as *mut u8, size_in_memory) };
        dest[..size_in_file].copy_from_slice(&data[offset..offset + size_in_file]);
        dest[size_in_file..].fill(0);
    }

    Ok((elf.entry, dest_start, num_pages))
}
