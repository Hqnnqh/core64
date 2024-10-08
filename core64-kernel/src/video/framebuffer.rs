use core::{fmt::Debug, ptr::write_volatile};

use core64_util::graphics::{
    Color,
    framebuffer::{BPP, FrameBufferMetadata},
};

use crate::video::VideoError;

/// Directly accesses video memory in order to display graphics
#[derive(Clone, Debug)]
pub(crate) struct RawFrameBuffer {
    pub(in crate::video) meta_data: FrameBufferMetadata,
}

impl RawFrameBuffer {
    /// Draws a pixel onto the screen at coordinates x,y and with the specified color. Returns, whether the action succeeds or the coordinates are invalid.
    pub(in crate::video) fn draw_pixel(
        &self,
        x: usize,
        y: usize,
        color: Color,
    ) -> Result<(), VideoError> {
        if !self.in_bounds(x, y) {
            return Err(VideoError::CoordinatesOutOfBounds(x, y));
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

        Ok(())
    }
    /// Fills entire display with certain color
    pub(crate) fn fill(&self, color: Color) {
        for x in 0..self.meta_data.width {
            for y in 0..self.meta_data.height {
                self.draw_pixel(x, y, color).unwrap();
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
