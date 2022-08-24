use std::collections::HashMap;

use thiserror::Error;

use crate::{FragmentFlip, FragmentResolution, GeneralResolution};

#[derive(Debug, Error)]
pub enum FragmentFinderError {
    #[error("Resolution for image {0} invalid")]
    InvalidResolution(usize),
    #[error("A maximum of 65536 can be handled. There are {0} used image.")]
    TooMuchImage(usize),
    #[error("The image {0} has a too big resolution")]
    ImageTooBig(usize),
}

#[derive(PartialEq, Eq, Debug)]
pub struct FragmentUse {
    pub x: i32,
    pub y: i32,
    pub image_id: u16,
    pub flip: FragmentFlip,
}

/// The output of [`find_fragments_in_images`].
/// The fragment (here) are 8×8 pixel of size.
/// A tile may be Flip on either or both axis (or not at all). Only the smallest of the 4 possible flip is added in this collection (based on the comparaison of the resulting pixels, as Rust compare u8 arrays).
/// The key is the pixels of the fragment (image are stored line by line, from top-left to bottom-right)
/// The key is where they are used. The x or y may be negative.
pub struct FragmentFinderData {
    pub collected: HashMap<[u8; 64], Vec<FragmentUse>>,
}

/// Find all 8×8 fragment all input images contain.
/// See [`FragmentFinderData`] for more information on the output
/// The image is filled on all sides (7 pixels) by 0s. Fragments consisting of only zeroes are discared.
/// 0×0 images are skipped.
pub fn find_fragments_in_images(
    images: &[(&[u8], GeneralResolution)],
) -> Result<FragmentFinderData, FragmentFinderError> {
    if images.len() > u16::MAX as usize {
        return Err(FragmentFinderError::TooMuchImage(images.len()));
    }
    let mut result = FragmentFinderData {
        collected: HashMap::new(),
    };
    let mut fragment_buffer = [0; 64];
    let mut fragment_buffer_horizontal = [0; 64];
    let mut fragment_buffer_vertical = [0; 64];
    let mut fragment_buffer_both = [0; 64];
    let fragment_resolution = FragmentResolution::new(8, 8);
    let zero_buffer = [0; 64];
    for (image_id, (image_pixels, resolution)) in images.iter().enumerate() {
        if image_pixels.len() as u64 != resolution.nb_pixels() {
            return Err(FragmentFinderError::InvalidResolution(image_id));
        };
        if image_pixels.is_empty() {
            continue;
        };
        let (padded_image, padded_resolution) =
            pad_seven_pixel(image_pixels, resolution.clone()).unwrap();
        //no panic: checked just before the resolution is good
        for x_base in 0..padded_resolution.x - 7 {
            for y_base in 0..padded_resolution.y - 7 {
                for special_line in 0..8 {
                    let pixel_base = (special_line + y_base) * padded_resolution.x + x_base;
                    fragment_buffer[special_line as usize * 8..special_line as usize * 8 + 8]
                        .copy_from_slice(
                            &padded_image[pixel_base as usize..pixel_base as usize + 8],
                        );
                }
                // collected a 8×8 fragment
                if fragment_buffer == zero_buffer {
                    continue;
                }
                // no unwrap: static valid resolution
                (FragmentFlip::FlipHorizontal)
                    .apply(
                        &fragment_buffer,
                        fragment_resolution,
                        &mut fragment_buffer_horizontal,
                    )
                    .unwrap();
                (FragmentFlip::FlipVertical)
                    .apply(
                        &fragment_buffer,
                        fragment_resolution,
                        &mut fragment_buffer_vertical,
                    )
                    .unwrap();
                (FragmentFlip::FlipBoth)
                    .apply(
                        &fragment_buffer,
                        fragment_resolution,
                        &mut fragment_buffer_both,
                    )
                    .unwrap();
                let mut smallest = &fragment_buffer;
                let mut smallest_flip = FragmentFlip::Standard;
                for (other_buffer, other_flip) in [
                    (&fragment_buffer_horizontal, FragmentFlip::FlipHorizontal),
                    (&fragment_buffer_vertical, FragmentFlip::FlipVertical),
                    (&fragment_buffer_both, FragmentFlip::FlipBoth),
                ] {
                    if other_buffer < smallest {
                        smallest = other_buffer;
                        smallest_flip = other_flip;
                    };
                }
                result
                    .collected
                    .entry(*smallest)
                    .or_default()
                    .push(FragmentUse {
                        x: x_base as i32 - 7,
                        y: y_base as i32 - 7,
                        // no overflow: already checked at the beggining of the function
                        image_id: image_id as u16,
                        flip: smallest_flip,
                    });
            }
        }
    }
    Ok(result)
}

fn pad_seven_pixel(
    image: &[u8],
    resolution: GeneralResolution,
) -> Option<(Vec<u8>, GeneralResolution)> {
    if image.len() != resolution.nb_pixels() as usize {
        return None;
    }
    let result_resolution = GeneralResolution::new(resolution.x + 14, resolution.y + 14);
    let mut result_px = Vec::with_capacity(result_resolution.nb_pixels() as usize);
    result_px.resize(result_resolution.x as usize * 7, 0);
    for line in image.chunks_exact(resolution.x as usize) {
        result_px.extend_from_slice(&[0; 7]);
        result_px.extend_from_slice(line);
        result_px.extend_from_slice(&[0; 7]);
    }
    result_px.resize(result_px.len() + (result_resolution.x as usize * 7), 0);
    Some((result_px, result_resolution))
}

#[test]
fn test_pad_seven_pixel() {
    let image = [2, 3, 4, 5, 6, 7];
    let mut expected_result = Vec::new();
    for _ in 0..(7 + 7 + 2) * 7 {
        expected_result.push(0);
    }
    expected_result.extend([
        0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 5, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 7, 0, 0, 0, 0, 0, 0, 0,
    ]);
    for _ in 0..(7 + 7 + 2) * 7 {
        expected_result.push(0);
    }
    assert_eq!(
        pad_seven_pixel(&image, GeneralResolution::new(2, 3)).unwrap(),
        (
            expected_result,
            GeneralResolution::new(7 + 7 + 2, 7 + 7 + 3)
        )
    );
}

#[cfg(test)]
mod tests {
    use crate::{
        find_fragments_in_images, fragment_finder::FragmentUse, FragmentFlip, GeneralResolution,
    };

    #[test]
    pub fn test_small_image() {
        assert_eq!(
            find_fragments_in_images(&[(&[], GeneralResolution::new(0, 0))])
                .unwrap()
                .collected
                .len(),
            0
        );

        let small_image = [1, 2, 1, 2, 1, 2, 1, 5];
        let found =
            find_fragments_in_images(&[(&small_image, GeneralResolution::new(2, 4))]).unwrap();
        let fragment_first = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1,
        ];

        assert!(found
            .collected
            .get(&fragment_first)
            .unwrap()
            .contains(&FragmentUse {
                x: -7,
                y: -7,
                image_id: 0,
                flip: FragmentFlip::Standard
            }));
        assert!(found
            .collected
            .get(&fragment_first)
            .unwrap()
            .contains(&FragmentUse {
                x: -7,
                y: 3,
                image_id: 0,
                flip: FragmentFlip::FlipHorizontal
            }));
        assert_eq!(found.collected.get(&fragment_first).unwrap().len(), 2);
    }

    #[test]
    pub fn test_ignore_zeroes() {
        let would_contain_zeroes = [0, 0, 0, 0, 1, 0, 0, 0, 0];
        assert!(
            find_fragments_in_images(&[(&would_contain_zeroes, GeneralResolution::new(3, 3))])
                .unwrap()
                .collected
                .get(&[0; 64])
                .is_none()
        );
    }
}
