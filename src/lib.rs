mod chunks;
mod error;

use chunks::ChunkWriter;
pub use error::PngError;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Seek, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    Grayscale,
    Rgb,
    GrayscaleAlpha,
    Rgba,
    Indexed,
}

impl ColorType {
    fn png_header_code(&self) -> u8 {
        match self {
            ColorType::Grayscale => 0,
            ColorType::Rgb => 2,
            ColorType::GrayscaleAlpha => 4,
            ColorType::Rgba => 6,
            ColorType::Indexed => 3,
        }
    }

    fn bytes_per_pixel(&self) -> usize {
        match self {
            ColorType::Grayscale => 1,
            ColorType::Rgb => 3,
            ColorType::GrayscaleAlpha => 2,
            ColorType::Rgba => 4,
            ColorType::Indexed => 1,
        }
    }

    fn validate_components(&self, components: &[u8]) -> Result<(), PngError> {
        let expected = self.bytes_per_pixel();
        if components.len() != expected {
            Err(PngError::ComponentCountMismatch {
                expected,
                actual: components.len(),
                color_type: *self,
            })
        } else {
            Ok(())
        }
    }
}

pub struct PngImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    color_type: ColorType,
    palette: Option<Vec<u8>>,
}

impl PngImage {
    pub fn new(width: u32, height: u32, color_type: ColorType) -> Result<Self, PngError> {
        if width == 0 || height == 0 || width > 0x7FFF || height > 0x7FFF {
            return Err(PngError::InvalidDimensions(width, height));
        }

        Ok(Self {
            width,
            height,
            data: Vec::with_capacity(
                (width as usize) * (height as usize) * color_type.bytes_per_pixel(),
            ),
            color_type,
            palette: None,
        })
    }

    pub fn add_pixel(&mut self, components: &[u8]) -> Result<(), PngError> {
        self.color_type.validate_components(components)?;

        // Check pixel count
        let max_pixels = (self.width * self.height) as usize;
        let current_pixels = self.data.len() / self.color_type.bytes_per_pixel();
        if current_pixels >= max_pixels {
            return Err(PngError::PixelCountMismatch {
                expected: max_pixels,
                actual: current_pixels + 1,
                dimensions: (self.width, self.height),
            });
        }

        self.data.extend_from_slice(components);
        Ok(())
    }

    fn generate_ihdr(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(13);

        data.extend_from_slice(&self.width.to_be_bytes());
        data.extend_from_slice(&self.height.to_be_bytes());

        // Bit depth (8 bits per sample)
        data.push(8);

        // Color type
        data.push(self.color_type.png_header_code());

        // Compression method (0 = DEFLATE)
        data.push(0);
        // Filter method (0 = adaptive filtering)
        data.push(0);
        // Interlace method (0 = none)
        data.push(0);

        data
    }

    fn filter_scanlines(&self) -> Vec<u8> {
        let bytes_per_pixel = self.color_type.bytes_per_pixel();
        let row_length = self.width as usize * bytes_per_pixel;
        let mut filtered = Vec::with_capacity(self.data.len() + self.height as usize);

        for row in self.data.chunks_exact(row_length) {
            filtered.push(1);

            let mut prev = vec![0; bytes_per_pixel];
            for (i, &byte) in row.iter().enumerate() {
                let channel = i % bytes_per_pixel;
                let filtered_byte = byte.wrapping_sub(prev[channel]);
                filtered.push(filtered_byte);
                prev[channel] = byte;
            }
        }
        filtered
    }

    pub fn write_to_file<W: Write + Seek>(&self, writer: &mut W) -> Result<(), PngError> {
        if self.color_type == ColorType::Indexed {
            if self.palette.is_none() {
                return Err(PngError::InvalidPalette(
                    "Palette required for indexed color".to_string(),
                ));
            }
            self.validate_palette_indices()?;
        }

        // Write PNG signature
        writer.write_all(&[137, 80, 78, 71, 13, 10, 26, 10])?;

        // Write IHDR chunk
        let ihdr_data = self.generate_ihdr();
        ChunkWriter::write_chunk(writer, b"IHDR", &ihdr_data)?;

        if let Some(palette) = &self.palette {
            ChunkWriter::write_chunk(writer, b"PLTE", palette)?;
        }

        // Process image data
        let filtered = self.filter_scanlines();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&filtered)?;
        let compressed = encoder.finish()?;
        ChunkWriter::write_chunk(writer, b"IDAT", &compressed)?;
        ChunkWriter::write_chunk(writer, b"IEND", &[])?;

        Ok(())
    }

    pub fn set_palette(&mut self, palette: &[u8]) -> Result<(), PngError> {
        if self.color_type != ColorType::Indexed {
            return Err(PngError::ColorTypeError);
        }

        if palette.len() % 3 != 0 {
            return Err(PngError::InvalidPalette(
                "Palette must contain RGB triplets".to_string(),
            ));
        }

        if palette.len() > 256 * 3 {
            return Err(PngError::InvalidPalette(
                "Palette cannot exceed 256 entries".to_string(),
            ));
        }

        self.palette = Some(palette.to_vec());
        Ok(())
    }

    fn validate_palette_indices(&self) -> Result<(), PngError> {
        if let Some(palette) = &self.palette {
            let max_index = (palette.len() / 3).saturating_sub(1);
            for &index in &self.data {
                if index as usize > max_index {
                    return Err(PngError::InvalidPaletteEntry(index));
                }
            }
        }
        Ok(())
    }
}
