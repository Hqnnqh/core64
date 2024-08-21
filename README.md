# Core64

Core64 is a foundational structure for building an x86-64 operating system in Rust. It includes the following components:

- **Custom Bootloader**: A UEFI application that sets up the environment for the kernel.
- **Minimal 64-bit Kernel Entry**: A simple kernel entry point to kickstart your OS development.
- **Util Crate**: A utility crate that provides shared functionality between different parts of the OS.

This project is ideal for those who want to learn or build a custom operating system from scratch using Rust.

## Components

1. **Bootloader**:
    - A UEFI-based bootloader that initializes the system and loads the kernel.
    - Sets up the 64-bit environment required for the kernel to run.
    - Enables basic paging to be able to run the higher half kernel

2. **Kernel**:
    - A minimal kernel entry point written in Rust.
    - Provides the groundwork for further kernel development.
    - Located at the higher half of the virtual address space

3. **Util Crate**:
    - Contains shared utilities and functions that are common across the kernel and loader.
    - Helps in keeping the codebase clean and modular.

## Example Usage

For an example of how to extend and use Core64 as a base for your operating system, check out the [ChickenOS project](https://github.com/chickensoftware/os).

## Next Steps
Core64 is just a minimal entry point for a kernel, and it currently lacks many of the essential features needed for a fully functioning operating system. As you continue to develop your OS, you'll need to implement memory management for the kernel itself, a gdt, interrupt handling, and many other features. Additionally, the loader itself should have proper error handling and display each step of the environment setup.

## OS Resources

If you're looking to expand your knowledge or find more tools and examples for OS development, here are some valuable resources:

- [OSDev Wiki](https://wiki.osdev.org/Main_Page): A comprehensive resource for operating system development.
- [dreamportdev's OsDev-Notes](https://github.com/dreamportdev/Osdev-Notes): A great collection of notes and guides on various aspects of developing an operating system.
- [Writing an OS in Rust](https://os.phil-opp.com/): A popular blog series that guides you through writing an OS in Rust. It's recommended to look at both version 1 and 2 since they do differ quite a bit.

## Getting Started

To get started with Core64:

Clone the repository:
   ```sh
   git clone https://github.com/Hqnnqh/core64.git
   ``` 
   
Boot Core64 in QEMU:
 ```sh
   make run release=true
``` 
or

Boot Core64 on a real machine using an USB:
```sh
   make usb USB_DEVICE=/dev/<device> release=true
``` 

A successful jump to the higher half kernel entry is indicated by the screen turning green.