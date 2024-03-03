use thiserror::Error;

use crate::GeneralResolution;

pub const FLIP_STANDARD: FragmentFlip = FragmentFlip::standard();
pub const FLIP_VERTICAL: FragmentFlip = FragmentFlip::vertical();
pub const FLIP_HORIZONTAL: FragmentFlip = FragmentFlip::horizontal();
pub const FLIP_BOTH: FragmentFlip = FragmentFlip::both();

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FragmentFlipError {
    #[error("Incoherent resolution")]
    IncoherentResolution,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct FragmentFlip {
    pub flip_h: bool,
    pub flip_v: bool,
}

impl FragmentFlip {
    #[inline(always)]
    pub const fn standard() -> Self {
        Self::from_bools(false, false)
    }

    #[inline(always)]
    pub const fn vertical() -> Self {
        Self::from_bools(true, false)
    }

    #[inline(always)]
    pub const fn horizontal() -> Self {
        Self::from_bools(false, true)
    }

    #[inline(always)]
    pub const fn both() -> Self {
        Self::from_bools(true, true)
    }

    /// Flip a tile with the corresponding flip value
    /// source and target are image using an u8 per pixel, row by row, from top-left most to bottom-right.
    /// target and source should have the correct number of pixel. Otherwise, and error is returned.
    pub fn apply(
        self,
        source: &[u8],
        resolution: GeneralResolution,
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
            FLIP_STANDARD => {
                // no panic: both have the number of pixels corresponding to the resolution
                target.copy_from_slice(source);
            }
            FLIP_HORIZONTAL => {
                // no panic: checked for 0 pixel list before. To have at least one pixel, x should be at least one
                for (source_chunk, target_chunk) in source
                    .chunks_exact(resolution.x as usize)
                    .zip(target.rchunks_exact_mut(resolution.x as usize))
                {
                    target_chunk.copy_from_slice(source_chunk);
                }
            }
            FLIP_VERTICAL => {
                // no panic: as before
                for (source_chunk, target_chunk) in source
                    .chunks_exact(resolution.x as usize)
                    .zip(target.chunks_exact_mut(resolution.x as usize))
                {
                    target_chunk
                        .copy_from_slice(&source_chunk.iter().copied().rev().collect::<Vec<u8>>());
                }
            }
            FLIP_BOTH => {
                target.copy_from_slice(&source.iter().copied().rev().collect::<Vec<u8>>());
            }
        };
        Ok(())
    }

    /// Return the [`FragmentFlip`] that would result in this flip applied to another one
    pub fn flipped_fragment(self, other: FragmentFlip) -> FragmentFlip {
        Self {
            flip_h: self.flip_h ^ other.flip_h,
            flip_v: self.flip_v ^ other.flip_v,
        }
    }

    /// Return a pair of boolean. First boolean tell if it should be vertically flipped, second one if it should be horizontally flipped
    pub const fn to_bools(self) -> (bool, bool) {
        (self.flip_v, self.flip_h)
    }

    /// Return the corresponding [`FragmentFlip`]. First boolean for vertical flip, second boolean for horizontal flip.
    #[inline(always)]
    pub const fn from_bools(flip_v: bool, flip_h: bool) -> FragmentFlip {
        Self { flip_h, flip_v }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FragmentFlip, FragmentFlipError, GeneralResolution};

    #[test]
    fn test_tile_flip_apply() {
        let test_data_4x4 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 4, 3, 2, 1];
        let resolution = GeneralResolution::new(4, 4);
        let mut target_4x4 = [0; 16];
        (FragmentFlip::standard())
            .apply(&test_data_4x4, resolution.clone(), &mut target_4x4)
            .unwrap();
        assert_eq!(target_4x4, test_data_4x4);

        (FragmentFlip::horizontal())
            .apply(&test_data_4x4, resolution.clone(), &mut target_4x4)
            .unwrap();
        assert_eq!(
            target_4x4,
            [4, 3, 2, 1, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3]
        );

        (FragmentFlip::vertical())
            .apply(&test_data_4x4, resolution.clone(), &mut target_4x4)
            .unwrap();
        assert_eq!(
            target_4x4,
            [3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 1, 2, 3, 4]
        );

        let mut target_3x3 = [0; 9];
        let test_data_3x3 = [3, 2, 1, 14, 13, 12, 5, 2, 6];
        (FragmentFlip::both())
            .apply(
                &test_data_3x3,
                GeneralResolution::new(3, 3),
                &mut target_3x3,
            )
            .unwrap();
        assert_eq!(target_3x3, [6, 2, 5, 12, 13, 14, 1, 2, 3]);

        assert_eq!(
            (FragmentFlip::vertical()).apply(
                &test_data_4x4,
                GeneralResolution::new(3, 3),
                &mut target_3x3
            ),
            Err(FragmentFlipError::IncoherentResolution)
        );

        assert_eq!(
            (FragmentFlip::vertical()).apply(
                &test_data_3x3,
                GeneralResolution::new(3, 3),
                &mut target_4x4
            ),
            Err(FragmentFlipError::IncoherentResolution)
        );

        (FragmentFlip::vertical())
            .apply(
                &test_data_4x4,
                GeneralResolution::new(2, 8),
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
            (FragmentFlip::standard(), (false, false)),
            (FragmentFlip::horizontal(), (false, true)),
            (FragmentFlip::vertical(), (true, false)),
            (FragmentFlip::both(), (true, true)),
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
