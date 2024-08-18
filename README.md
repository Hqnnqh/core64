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

## Getting Started

To get started with Core64:

1. Clone the repository:
   ```sh
   git clone https://github.com/Hqnnqh/core64.git
