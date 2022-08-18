use thiserror::Error;

use crate::FragmentResolution;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FragmentFlipError {
    #[error("Non square resolution")]
    NonSquareResolution,
    #[error("Incoherent resolution")]
    IncoherentResolution,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FragmentFlip {
    Standard,
    FlipHorizontal,
    FlipVertical,
    FlipBoth,
}

impl FragmentFlip {
    /// Flip a tile with the corresponding flip value
    /// source and target are image using an u8 per pixel, row by row, from top-left most to bottom-right.
    /// target and source should have the correct number of pixel. Otherwise, and error is returned.
    pub fn apply(
        &self,
        source: &[u8],
        resolution: FragmentResolution,
        target: &mut [u8],
    ) -> Result<(), FragmentFlipError> {
        if resolution.nb_pixels() as usize != source.len()
            || resolution.nb_pixels() as usize != target.len()
        {
            return Err(FragmentFlipError::IncoherentResolution);
        };
        if resolution.nb_pixels() == 0 {
            return Ok(());
        };
        match self {
            Self::Standard => {
                // no panic: both have the number of pixels corresponding to the resolution
                target.copy_from_slice(source);
            }
            Self::FlipHorizontal => {
                // no panic: checked for 0 pixel list before. To have at least one pixel, x should be at least one
                for (source_chunk, target_chunk) in source
                    .chunks_exact(resolution.x as usize)
                    .zip(target.rchunks_exact_mut(resolution.x as usize))
                {
                    target_chunk.copy_from_slice(source_chunk);
                }
            }
            Self::FlipVertical => {
                // no panic: as before
                for (source_chunk, target_chunk) in source
                    .chunks_exact(resolution.x as usize)
                    .zip(target.chunks_exact_mut(resolution.x as usize))
                {
                    target_chunk
                        .copy_from_slice(&source_chunk.iter().copied().rev().collect::<Vec<u8>>());
                }
            }
            Self::FlipBoth => {
                target.copy_from_slice(&source.iter().copied().rev().collect::<Vec<u8>>());
            }
        };
        Ok(())
    }

    // Return a pair of boolean. First boolean tell if it should be vertically flipped, second one if it should be horizontally flipped
    pub fn to_bools(self) -> (bool, bool) {
        match self {
            Self::Standard => (false, false),
            Self::FlipVertical => (true, false),
            Self::FlipHorizontal => (false, true),
            Self::FlipBoth => (true, true),
        }
    }

    // Return the corresponding [`FragmentFlip`]. First boolean for vertical flip, second boolean for horizontal flip.
    pub fn from_bools(flip_v: bool, flip_h: bool) -> FragmentFlip {
        match (flip_v, flip_h) {
            (false, false) => Self::Standard,
            (true, false) => Self::FlipVertical,
            (false, true) => Self::FlipHorizontal,
            (true, true) => Self::FlipBoth,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FragmentFlip, FragmentFlipError, FragmentResolution};

    #[test]
    fn test_tile_flip_apply() {
        let test_data_4x4 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 4, 3, 2, 1];
        let resolution = FragmentResolution::new(4, 4);
        let mut target_4x4 = [0; 16];
        (FragmentFlip::Standard)
            .apply(&test_data_4x4, resolution, &mut target_4x4)
            .unwrap();
        assert_eq!(target_4x4, test_data_4x4);

        (FragmentFlip::FlipHorizontal)
            .apply(&test_data_4x4, resolution, &mut target_4x4)
            .unwrap();
        assert_eq!(
            target_4x4,
            [4, 3, 2, 1, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3]
        );

        (FragmentFlip::FlipVertical)
            .apply(&test_data_4x4, resolution, &mut target_4x4)
            .unwrap();
        assert_eq!(
            target_4x4,
            [3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 1, 2, 3, 4]
        );

        let mut target_3x3 = [0; 9];
        let test_data_3x3 = [3, 2, 1, 14, 13, 12, 5, 2, 6];
        (FragmentFlip::FlipBoth)
            .apply(
                &test_data_3x3,
                FragmentResolution::new(3, 3),
                &mut target_3x3,
            )
            .unwrap();
        assert_eq!(target_3x3, [6, 2, 5, 12, 13, 14, 1, 2, 3]);

        assert_eq!(
            (FragmentFlip::FlipVertical).apply(
                &test_data_4x4,
                FragmentResolution::new(3, 3),
                &mut target_3x3
            ),
            Err(FragmentFlipError::IncoherentResolution)
        );

        assert_eq!(
            (FragmentFlip::FlipVertical).apply(
                &test_data_3x3,
                FragmentResolution::new(3, 3),
                &mut target_4x4
            ),
            Err(FragmentFlipError::IncoherentResolution)
        );

        (FragmentFlip::FlipVertical)
            .apply(
                &test_data_4x4,
                FragmentResolution::new(2, 8),
                &mut target_4x4,
            )
            .unwrap();

        assert_eq!(
            target_4x4,
            [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 3, 4, 1, 2]
        )
    }

    #[test]
    fn fragment_flip_convert_to_from_boolean() {
        let expected = [
            (FragmentFlip::Standard, (false, false)),
            (FragmentFlip::FlipHorizontal, (false, true)),
            (FragmentFlip::FlipVertical, (true, false)),
            (FragmentFlip::FlipBoth, (true, true)),
        ];
        for (fragment_flip, boolean_flip) in &expected {
            assert_eq!(&fragment_flip.to_bools(), boolean_flip);
            assert_eq!(
                &FragmentFlip::from_bools(boolean_flip.0, boolean_flip.1),
                fragment_flip
            )
        }
    }
}
