use std::collections::BTreeMap;

use thiserror::Error;

use crate::{FragmentFlip, GeneralResolution, NormalizedBytes};

#[derive(Debug, Error)]
pub enum FragmentFinderError {
    #[error("Resolution for image {0} invalid")]
    InvalidResolution(usize),
    #[error("A maximum of 65536 can be handled. There are {0} used image.")]
    TooMuchImage(usize),
    #[error("The image {0} has a too big resolution")]
    ImageTooBig(usize),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
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
#[derive(Default)]
pub struct FragmentFinderData {
    pub collected: BTreeMap<NormalizedBytes, Vec<FragmentUse>>,
}

impl FragmentFinderData {
    /// Add info about the usage of a fragment. Will add it if it already exist.
    pub fn add_fragment_use(&mut self, bytes: NormalizedBytes, usage: FragmentUse) {
        self.collected.entry(bytes).or_default().push(usage);
    }

    /// Return a list with element sorted by the number of time they appear (most used appear first)
    pub fn order_by_usage(&self) -> Vec<(&NormalizedBytes, &Vec<FragmentUse>)> {
        let mut r = self.collected.iter().collect::<Vec<_>>();
        r.sort_by_key(|x| usize::MAX - x.1.len());
        r
    }
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
        collected: BTreeMap::new(),
    };
    let mut fragment_buffer = [0; 64];
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
                let (normalized, flip) = NormalizedBytes::new(fragment_buffer);
                result.add_fragment_use(
                    normalized,
                    FragmentUse {
                        x: x_base as i32 - 7,
                        y: y_base as i32 - 7,
                        // no overflow: already checked at the beggining of the function
                        image_id: image_id as u16,
                        flip,
                    },
                );
            }
        }
    }
    Ok(result)
}

pub fn pad_seven_pixel(
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

#[cfg(test)]
mod tests {
    use crate::{
        find_fragments_in_images,
        fragment_finder::{pad_seven_pixel, FragmentUse},
        FragmentFinderData, FragmentFlip, GeneralResolution, NormalizedBytes,
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
        let (fragment_first, _) = NormalizedBytes::new([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1,
        ]);

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
                .get(&NormalizedBytes::new([0; 64]).0)
                .is_none()
        );
    }

    #[test]
    pub fn test_order_by_usage() {
        let mut bytes_present_twice = [0; 64];
        bytes_present_twice[0] = 1;
        let mut bytes_present_once = [0; 64];
        bytes_present_once[0] = 2;
        let mut fragment_finder_data = FragmentFinderData::default();

        for (counter, bytes) in [bytes_present_twice, bytes_present_once, bytes_present_twice]
            .iter()
            .enumerate()
        {
            let (normalized, flip) = NormalizedBytes::new(*bytes);
            fragment_finder_data.add_fragment_use(
                normalized,
                FragmentUse {
                    x: 0,
                    y: 0,
                    image_id: counter as u16,
                    flip,
                },
            );
        }

        let fragment_usage_ordered = fragment_finder_data.order_by_usage();
        assert_eq!(
            fragment_usage_ordered[0].0,
            &NormalizedBytes::new(bytes_present_twice).0
        );
        assert_eq!(
            fragment_usage_ordered[1].0,
            &NormalizedBytes::new(bytes_present_once).0
        );
    }

    #[test]
    fn test_pad_seven_pixel() {
        let image = [2, 3, 4, 5, 6, 7];
        let mut expected_result = Vec::new();
        for _ in 0..(7 + 7 + 2) * 7 {
            expected_result.push(0);
        }
        expected_result.extend([
            0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 5, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 7, 0, 0, 0, 0, 0, 0, 0,
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
}
