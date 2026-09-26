use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use super::canvas::Canvas;

/// Write the canvas as an RGB PNG, upscaled `k` times (nearest neighbor).
pub fn write(path: &Path, cv: &Canvas, k: usize) -> io::Result<()> {
    let mut rgb = Vec::new();
    cv.scaled_rgb(k, &mut rgb);
    let file = BufWriter::new(File::create(path)?);
    let mut enc = png::Encoder::new(file, (cv.w as usize * k) as u32, (cv.h as usize * k) as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(io::Error::other)?;
    w.write_image_data(&rgb).map_err(io::Error::other)?;
    Ok(())
}
