#![no_std]
#![no_main]

use core::panic::PanicInfo;

use core_util::{BootInfo, graphics::Color};

use crate::video::framebuffer::RawFrameBuffer;

mod video;

#[no_mangle]
pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> ! {
    let framebuffer = RawFrameBuffer::from(boot_info.frame_buffer_metadata);
    framebuffer.fill(Color::green());

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
