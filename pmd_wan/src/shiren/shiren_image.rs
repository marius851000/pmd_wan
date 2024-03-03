use anyhow::{bail, Context};
use image::{imageops, ImageBuffer, Rgba};

use super::{ShirenFragment, ShirenFragmentBytes, ShirenFrame, ShirenPalette, ShirenWan};

pub fn shiren_export_fragment(
    fragment: &ShirenFragment,
    fragment_bytes: &ShirenFragmentBytes,
    palette: &ShirenPalette,
) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    // TODO: make sure we select the good palette
    // TODO: vertical flip once located

    if palette.colors.len() < 0x10 {
        bail!("The input palette does not have at least 16 colors");
    }

    let resolution = fragment.oam_shape.size();
    let mut image = ImageBuffer::new(resolution.x as u32, resolution.y as u32);

    if resolution.nb_pixels()
        > (fragment_bytes.bytes.len() * 2)
            .try_into()
            .unwrap_or(u64::MAX)
    {
        bail!("The resolution {:?} for the fragment have {} pixel, but the fragment bytes only have size for {} pixels", resolution, resolution.nb_pixels(), fragment_bytes.bytes.len() * 2);
    }

    let mut iterator = fragment_bytes.bytes.iter().copied();

    fn transform_color(mut color: [u8; 4]) -> [u8; 4] {
        color[3] = color[3].checked_mul(2).unwrap_or(255);
        return color;
    }

    for chunk_y in 0..resolution.y / 8 {
        for chunk_x in 0..resolution.x / 8 {
            for y in 0..8 {
                for x_nb in 0..4 {
                    let byte = if let Some(byte) = iterator.next() {
                        byte
                    } else {
                        // This shouldn’t happen as we previously check the amount of of bytes match the amount of pixels
                        panic!();
                    };
                    let pixel_id_1 = ((byte & 0xF0) >> 4) + 0 * 16;
                    let pixel_id_2 = (byte & 0x0F) + 0 * 16;
                    let x1 = chunk_x * 8 + x_nb * 2;
                    let y1 = chunk_y * 8 + y;

                    if pixel_id_1 % 16 != 0 {
                        image.put_pixel(
                            x1 as u32 + 1,
                            y1 as u32,
                            Rgba::from(transform_color(palette.colors[pixel_id_1 as usize])),
                        );
                    }
                    if pixel_id_2 % 16 != 0 {
                        image.put_pixel(
                            x1 as u32,
                            y1 as u32,
                            Rgba::from(transform_color(palette.colors[pixel_id_2 as usize])),
                        );
                    }
                }
            }
        }
    }

    if fragment.is_h_flip {
        imageops::flip_horizontal_in_place(&mut image);
    }

    return Ok(image);
}

/// Result:
/// 1. The image assembling all the fragment from the frame
/// 2. The xy position of the “central” point of the palette, relative to the top-left of the result image
pub fn shiren_export_frame(
    frame: &ShirenFrame,
    wan_image: &ShirenWan,
    palette: &ShirenPalette,
) -> anyhow::Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, (usize, usize))> {
    // 1. Calculate the resolution of the animation frame
    let (mut x_max, mut x_min, mut y_max, mut y_min): (i32, i32, i32, i32) = (0, 0, 0, 0);
    for fragment in &frame.fragments {
        let (fragment_x_min, fragment_y_min): (i32, i32) =
            (fragment.offset_x.into(), fragment.offset_y.into());
        let (fragment_x_max, fragment_y_max): (i32, i32) = (
            //No overflow: OamShape.size() values are always <= 64
            fragment_x_min + fragment.oam_shape.size().x as i32,
            fragment_y_min + fragment.oam_shape.size().y as i32,
        );
        x_min = x_min.min(fragment_x_min);
        y_min = y_min.min(fragment_y_min);
        y_max = y_max.max(fragment_y_max);
        x_max = x_max.max(fragment_x_max);
    }
    // Make sure the image is not 0-sized (will only happen if there is not fragment to include)
    if x_max - x_min == 0 {
        x_max = 8;
    }
    if y_max - y_min == 0 {
        y_max = 8;
    }
    // 2. Actually assemble that frame
    let (x_offset, y_offset) = (-x_min as u32, -y_min as u32);
    let mut image = ImageBuffer::new(x_offset + x_max as u32, y_offset + y_max as u32);

    for (fragment_nb, fragment) in frame.fragments.iter().enumerate() {
        if let Some(fragment_bytes_id) = fragment.fragment_bytes_id {
            let fragment_bytes = wan_image
                .fragment_bytes_store
                .fragment_bytes
                .get(fragment_bytes_id as usize).with_context(|| format!("Attempting to index non-existant fragment bytes id {} for fragment number {}", fragment_bytes_id, fragment_nb))?;

            let fragment_image = shiren_export_fragment(fragment, fragment_bytes, palette)
                .with_context(|| {
                    format!(
                        "While reading the pixels of fragment number {}",
                        fragment_nb,
                    )
                })?;
            imageops::overlay(
                &mut image,
                &fragment_image,
                (fragment.offset_x + x_offset as i16) as i64,
                (fragment.offset_y + y_offset as i16) as i64,
            ); //TODO: the spritebot library doesn’t support out of bound center
        }
    }
    Ok((image, (x_offset as usize, y_offset as usize)))
}
