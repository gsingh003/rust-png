use std::fs::File;

use png::{ColorType, PngError, PngImage};

fn main() -> Result<(), PngError> {
    let img_width = 256;
    let img_height = 256;

    let mut img = PngImage::new(img_width, img_height, ColorType::Rgba)?;

    for y in 0..256 {
        for x in 0..256 {
            let r: f64 = (y as f64) / (img_height as f64 - 1.0);
            let g: f64 = (x as f64) / (img_width as f64 - 1.0);
            let b = 0.0;

            let ir = (255.999 * r) as u8;
            let ig = (255.999 * g) as u8;
            let ib = (255.999 * b) as u8;

            img.add_pixel(&[ir, ig, ib, 255])?;
        }
    }

    let mut file = File::create("gradient.png")?;
    img.write_to_file(&mut file)?;
    create_grayscale_image()?;
    create_grayscale_alpha_image()?;
    create_indexed_image()?;

    Ok(())
}

fn create_grayscale_image() -> Result<(), PngError> {
    let mut img = PngImage::new(128, 128, ColorType::Grayscale)?;
    for y in 0..128 {
        for x in 0..128 {
            let intensity = ((x as f32 + y as f32) / 2.0) as u8;
            img.add_pixel(&[intensity])?;
        }
    }
    img.write_to_file(&mut File::create("grayscale.png")?)?;
    Ok(())
}

fn create_grayscale_alpha_image() -> Result<(), PngError> {
    let mut img = PngImage::new(64, 64, ColorType::GrayscaleAlpha)?;
    for y in 0..64 {
        for x in 0..64 {
            let intensity = (x + y) as u8;
            let alpha = (x * 4) as u8;
            img.add_pixel(&[intensity, alpha])?;
        }
    }
    img.write_to_file(&mut File::create("grayscale_alpha.png")?)?;
    Ok(())
}

fn create_indexed_image() -> Result<(), PngError> {
    let mut img = PngImage::new(8, 8, ColorType::Indexed)?;

    // create a 3-color palette (RGB triplets)
    let palette = vec![
        255, 0, 0, // Red
        0, 255, 0, // Green
        0, 0, 255, // Blue
    ];
    img.set_palette(&palette)?;

    // Create checkerboard pattern
    for y in 0..8 {
        for x in 0..8 {
            let index = ((x + y) % 3) as u8;
            img.add_pixel(&[index])?;
        }
    }

    img.write_to_file(&mut File::create("palette.png")?)?;
    Ok(())
}
