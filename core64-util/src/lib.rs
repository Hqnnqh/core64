#![no_std]

use crate::graphics::framebuffer::FrameBufferMetadata;

pub mod graphics;
pub mod memory;

#[derive(Clone, Debug)]
pub struct BootInfo {
    pub frame_buffer_metadata: FrameBufferMetadata,
}
