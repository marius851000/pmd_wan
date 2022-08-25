use crate::{FragmentFlip, FragmentResolution};

/// Represent a 8×8 bytes that is been normalized, as such that all four [`FragmentFlip`] will result in the same [`NormalizedBytes`]
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct NormalizedBytes(pub [u8; 64]);

impl NormalizedBytes {
    /// The returned [`FragmentFlip`] is the transformation that has been applied
    pub fn new(bytes: [u8; 64]) -> (Self, FragmentFlip) {
        let mut cache = [[0; 64]; 3];
        let fragment_resolution = FragmentResolution::new(8, 8);
        (FragmentFlip::FlipHorizontal)
            .apply(&bytes, fragment_resolution, &mut cache[0])
            .unwrap();
        (FragmentFlip::FlipVertical)
            .apply(&bytes, fragment_resolution, &mut cache[1])
            .unwrap();
        (FragmentFlip::FlipBoth)
            .apply(&bytes, fragment_resolution, &mut cache[2])
            .unwrap();
        let mut smallest = &bytes;
        let mut smallest_flip = FragmentFlip::Standard;
        for (other_buffer, other_flip) in [
            (&cache[0], FragmentFlip::FlipHorizontal),
            (&cache[1], FragmentFlip::FlipVertical),
            (&cache[2], FragmentFlip::FlipBoth),
        ] {
            if other_buffer < smallest {
                smallest = other_buffer;
                smallest_flip = other_flip;
            };
        }
        (Self(*smallest), smallest_flip)
    }
}

#[cfg(test)]
mod tests {
    use crate::{FragmentFlip, NormalizedBytes};

    #[test]
    fn test_normalized_bytes() {
        let mut base = [0; 64];
        base[63] = 1;
        let mut flipboth = [0; 64];
        flipboth[0] = 1;
        let mut flipvert = [0; 64];
        flipvert[63 - 7] = 1;
        let mut fliphor = [0; 64];
        fliphor[7] = 1;
        for (bytes, flip) in [
            (base, FragmentFlip::Standard),
            (flipboth, FragmentFlip::FlipBoth),
            (flipvert, FragmentFlip::FlipVertical),
            (fliphor, FragmentFlip::FlipHorizontal),
        ] {
            assert_eq!(NormalizedBytes::new(bytes), (NormalizedBytes(base), flip));
        }
    }
}
