use anyhow::bail;
use image::{ImageBuffer, Rgba};

use crate::FragmentResolution;

use super::{ShirenFragment, ShirenFragmentBytes, ShirenPalette};

pub fn shiren_export_fragment(_fragment: &ShirenFragment, fragment_bytes: &ShirenFragmentBytes, palette: &ShirenPalette) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    if palette.colors.len() < 0x10 {
        bail!("The input palette does not have at least 16 colors");
    }
    //TODO: check size
    let resolution = FragmentResolution::new(32, 32);
    let mut image = ImageBuffer::new(resolution.x as u32, resolution.y as u32);

    //TODO: error handling
    let mut iterator = fragment_bytes.bytes.iter().copied();
    
    fn transform_color(mut color: [u8; 4]) -> [u8; 4] {
        color[3] = color[3].checked_mul(2).unwrap_or(255);
        return color;
    }

    for chunk_y in 0..resolution.y/8 {
        for chunk_x in 0..resolution.x/8 {
            for y in 0..8 {
                for x_nb in 0..4 {
                    let byte = iterator.next().unwrap();
                    let pixel_id_1 = (byte & 0xF0 >> 4) + 0*16;
                    let pixel_id_2 = (byte & 0x0F) + 0*16;
                    let x1 = chunk_x * 8 + x_nb * 2;
                    let y1 = chunk_y * 8 + y;

                    image.put_pixel(x1 as u32, y1 as u32, Rgba::from(transform_color(palette.colors[pixel_id_1 as usize])));
                    image.put_pixel(x1 as u32 + 1, y1 as u32, Rgba::from(transform_color(palette.colors[pixel_id_2 as usize])));
                }
            }
        }
    }

    return Ok(image);
}