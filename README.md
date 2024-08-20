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
### **Base Setup**
- **Global Descriptor Table (GDT)**: Set up the GDT to manage different segments in your kernel, ensuring proper memory access and privilege levels.
- **Interrupt Handling**: Implement the Interrupt Descriptor Table (IDT) and handlers to manage hardware and software interrupts.
- **Input/Output (IO)**: Develop basic IO routines to interact with hardware devices, such as keyboard and screen.
- **Timer Setup**: Configure the system timer (e.g., PIT or APIC) for timekeeping and scheduling purposes.

### **Memory Management**
- **Paging**: Implement paging to manage memory mapping in the kernel itself, protecting and isolating different parts of your OS.
- **Virtual Memory Manager (VMM)**: Create a VMM to handle virtual memory allocations, mapping, and page table management.
- **Heap Management**: Set up a dynamic memory allocator (heap) for efficient memory allocation and deallocation in the kernel.
- **Memory Protection**: Implement memory protection mechanisms to prevent unauthorized access and ensure kernel stability.

### **Process and Task Management**
- **Process Management**: Implement basic process management to create, switch, and terminate processes in your OS.
- **Task Scheduler**: Develop a task scheduler to manage process execution, ensuring fair CPU time allocation.
- **Context Switching**: Implement context switching to save and restore process states during multitasking.
- **Inter-Process Communication (IPC)**: Set up IPC mechanisms for processes to communicate and synchronize their actions.

### **File System**
- **File System Interface**: Design a basic file system interface for reading and writing files.
- **Disk Drivers**: Develop disk drivers to interact with storage devices, such as hard drives or SSDs.
- **File Management**: Implement file management functions, including file creation, deletion, and directory handling.

### **Driver Development**
- **Device Drivers**: Write drivers for essential hardware components like keyboards, mice, and display adapters.
- **Driver Interface**: Create a generic driver interface for easy addition of new hardware support.

### **User Mode and Applications**
- **User Mode Transition**: Implement a transition from kernel mode to user mode to run user-space applications.
- **System Calls**: Develop system calls that user-space applications can use to interact with the kernel.
- **Application Loading**: Implement loading and execution of user-space applications.

### **Networking**
- **Network Stack**: Build a basic network stack to handle data transmission and reception over the network.
- **Network Drivers**: Develop drivers for network interfaces such as Ethernet or Wi-Fi.

These are just some of the directions you can take; feel free to explore and expand your operating system in any way you like.

## Getting Started

To get started with Core64:

1. Clone the repository:
   ```sh
   git clone https://github.com/Hqnnqh/core64.git
