use core::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

pub(crate) mod framebuffer;

#[derive(Copy, Clone)]
enum VideoError {
    CoordinatesOutOfBounds(usize, usize),
}

impl Debug for VideoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            VideoError::CoordinatesOutOfBounds(x, y) => write!(
                f,
                "Video Error: Coordinates out of bounds: x: {}, y: {}.",
                x, y
            ),
        }
    }
}

impl Display for VideoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for VideoError {}
