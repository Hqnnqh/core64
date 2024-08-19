use alloc::{format, string::String};

use uefi::{
    prelude::BootServices,
    proto::console::gop::{GraphicsOutput, PixelFormat},
};

use core_util::graphics::framebuffer::FrameBufferMetadata;

/// Initialize framebuffer (GOP)

pub(super) fn initialize_framebuffer(
    boot_services: &BootServices,
) -> Result<FrameBufferMetadata, String> {
    let gop_handle = boot_services
        .get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|error| format!("Could not get handle for GOP: {error}."))?;

    let mut gop = boot_services
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .map_err(|error| format!("Could not open GOP: {error}."))?;
    let mut raw_frame_buffer = gop.frame_buffer();
    let base = raw_frame_buffer.as_mut_ptr() as u64;
    let size = raw_frame_buffer.size();
    let info = gop.current_mode_info();

    let is_rgb = match info.pixel_format() {
        PixelFormat::Rgb => Ok(true),
        PixelFormat::Bgr => Ok(false),
        PixelFormat::Bitmask | PixelFormat::BltOnly => {
            Err("ChickenOS (for now) only supports RGB and BGR pixel formats!")
        }
    }?;
    let (width, height) = info.resolution();
    let stride = info.stride();

    Ok(FrameBufferMetadata {
        base,
        size,
        width,
        height,
        stride,
        is_rgb,
    })
}

pub mod testing_fb {
    use core::{fmt::Debug, ptr::write_volatile};

    use core_util::graphics::{
        Color,
        framebuffer::{BPP, FrameBufferMetadata},
    };

    /// Directly accesses video memory in order to display graphics
    #[derive(Clone, Debug)]
    pub(crate) struct RawFrameBuffer {
        pub meta_data: FrameBufferMetadata,
    }

    impl RawFrameBuffer {
        /// Draws a pixel onto the screen at coordinates x,y and with the specified color. Returns, whether the action succeeds or the coordinates are invalid.
        pub fn draw_pixel(&self, x: usize, y: usize, color: Color) -> bool {
            if !self.in_bounds(x, y) {
                return false;
            }

            let pitch = self.meta_data.stride * BPP;

            unsafe {
                let pixel = (self.meta_data.base as *mut u8).add(pitch * y + BPP * x);

                if self.meta_data.is_rgb {
                    write_volatile(pixel, color.red); // Red
                    write_volatile(pixel.add(1), color.green); // Green
                    write_volatile(pixel.add(2), color.blue); // Blue
                } else {
                    write_volatile(pixel, color.blue); // Blue
                    write_volatile(pixel.add(1), color.green); // Green
                    write_volatile(pixel.add(2), color.red); // Red
                }
            }
            true
        }
        /// Fills entire display with certain color
        pub(crate) fn fill(&self, color: Color) {
            for x in 0..self.meta_data.width {
                for y in 0..self.meta_data.height {
                    self.draw_pixel(x, y, color);
                }
            }
        }
    }

    impl RawFrameBuffer {
        /// Whether a point is within the framebuffer vram
        fn in_bounds(&self, x: usize, y: usize) -> bool {
            x < self.meta_data.width && y < self.meta_data.height
        }
    }

    impl From<FrameBufferMetadata> for RawFrameBuffer {
        fn from(value: FrameBufferMetadata) -> Self {
            Self { meta_data: value }
        }
    }
}
