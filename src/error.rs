use std::io;
use thiserror::Error;

use crate::ColorType;

#[derive(Error, Debug)]
pub enum PngError {
    #[error("Invalid image dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid color type configuration")]
    ColorTypeError,

    #[error("Invalid component count: expected {expected} for {color_type:?}, got {actual}")]
    ComponentCountMismatch {
        expected: usize,
        actual: usize,
        color_type: ColorType,
    },

    #[error("Invalid pixel count: expected {expected} ({}x{}), got {actual}", .dimensions.0, .dimensions.1)]
    PixelCountMismatch {
        expected: usize,
        actual: usize,
        dimensions: (u32, u32),
    },

    #[error("Invalid palette: {0}")]
    InvalidPalette(String),

    #[error("Invalid palette index: {0}")]
    InvalidPaletteEntry(u8),
}

impl From<flate2::CompressError> for PngError {
    fn from(e: flate2::CompressError) -> Self {
        PngError::Compression(e.to_string())
    }
}
