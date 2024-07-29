use crate::GeneralResolution;

// The index is calculated from shape_indice << 2 + size_indice
static INDICE_TO_SIZE_MAP: [Option<(u8, u8)>; 16] = [
    Some((8, 8)),
    Some((16, 16)),
    Some((32, 32)),
    Some((64, 64)),
    Some((16, 8)),
    Some((32, 8)),
    Some((32, 16)),
    Some((64, 32)),
    Some((8, 16)),
    Some((8, 32)),
    Some((16, 32)),
    Some((32, 64)),
    None,
    None,
    None,
    None
];

/// One of the possible shape usable by the DS’s OAM
/// See LCD OBJ - OAM Attributes of GBATEK.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OamShape {
    // Make sure both of them are valid when setting them.
    shape_indice: u8,
    size_indice: u8
}

impl OamShape {
    pub fn new(shape_indice: u8, size_indice: u8) -> Option<Self> {
        if shape_indice <= 2 && size_indice <= 3 {
            Some(Self {
                shape_indice,
                size_indice
            })
        } else {
            None
        }
    }

    pub fn shape_indice(&self) -> u8 {
        self.shape_indice
    }

    pub fn size_indice(&self) -> u8 {
        self.size_indice
    }

    pub fn size(&self) -> GeneralResolution {
        let indice = (self.shape_indice << 2) + self.size_indice;
        let size_raw = INDICE_TO_SIZE_MAP[indice as usize].unwrap(); // No unwrap: it is always checked that the values are inside the valid range.
        GeneralResolution::new(size_raw.0.into(), size_raw.1.into())
    }

    pub fn chunk_to_allocate_for_fragment(&self) -> u16 {
        let size = self.size();
        let blocks = (size.x * size.y / 256) as u16;
        if blocks >= 1 {blocks} else {1}
    }

    /// Return the smallest resolution (in term of allocation) that can contain the target resolution.
    ///
    /// If there are multiple posible resolution with the same number of size to allocate, returnt the one with the lesser amount of pixel. If there are still multiple remaining resolution, return any possible one (implementation detail: they aren't random).
    pub fn find_smallest_containing(
        target_resolution: GeneralResolution,
    ) -> Option<OamShape> {
        let mut optimal_result: Option<(u16, u16, OamShape)> = None; // first u16 is number of chunk to allocate for the frame, second u16 is the number of pixel, third is the optimal resolution right now
        for (indice_compressed, entry) in INDICE_TO_SIZE_MAP.iter().copied().enumerate() {
            if let Some(entry) = entry {
                let resolution_entry = GeneralResolution::new(entry.0.into(), entry.1.into());
                if resolution_entry.can_contain(target_resolution.clone()) {
                    let entry_oam = OamShape {
                        shape_indice: (indice_compressed as u8) >> 2,
                        size_indice: (indice_compressed as u8) & 0b11
                    };
                    let chunk_to_allocate_entry = entry_oam.chunk_to_allocate_for_fragment();
                    let pixel_nb_entry = (resolution_entry.x as u16) * (resolution_entry.y as u16);
                    if let Some((chunk_to_allocate_optimal, pixel_nb_optimal, _)) = &optimal_result {
                        if *chunk_to_allocate_optimal > chunk_to_allocate_entry
                            || (*chunk_to_allocate_optimal == chunk_to_allocate_entry
                                && *pixel_nb_optimal > pixel_nb_entry)
                        {
                            optimal_result =
                                Some((chunk_to_allocate_entry, pixel_nb_entry, entry_oam));
                        }
                    } else {
                        optimal_result =
                            Some((chunk_to_allocate_entry, pixel_nb_entry, entry_oam));
                    };
                }
            }
        }
        optimal_result.map(|x| x.2)
    }
}

//TODO: check by decoding every images too
#[cfg(test)]
mod tests {
    use crate::{GeneralResolution, OamShape};
    #[test]
    fn test_resolution_chunk_allocation() {
        for ((shape, size), expected_output) in [((0, 2), 4), ((1, 1), 2), ((0, 3), 16)].into_iter()
        {
            let resolution = OamShape::new(shape, size).unwrap();
            let got = resolution.chunk_to_allocate_for_fragment();
            if got != expected_output {
                panic!(
                    "The resolution {:?} return the allocation number {}, but {} were expected",
                    resolution, got, expected_output
                );
            }
        }
    }

    #[test]
    pub fn test_creation_fail() {
        assert_eq!(OamShape::new(2, 4), None);
        assert_eq!(OamShape::new(3, 2), None);
        assert_eq!(OamShape::new(12, 255), None);
    }

    #[test]
    pub fn test_size() {
        assert_eq!(OamShape::new(0, 3).unwrap().size(), GeneralResolution::new(64, 64));
        assert_eq!(OamShape::new(1, 2).unwrap().size(), GeneralResolution::new(32, 16));
        assert_eq!(OamShape::new(2, 3).unwrap().size(), GeneralResolution::new(32, 64));
    }

    #[test]
    pub fn test_find_smaller() {
        assert_eq!(
            OamShape::find_smallest_containing(GeneralResolution::new(6, 3)).unwrap(),
            OamShape::new(0, 0).unwrap() // 8 by 8
        );
        assert_eq!(
            OamShape::find_smallest_containing(GeneralResolution::new(64, 10)).unwrap(),
            OamShape::new(1, 3).unwrap() // 64 by 32
        );
        let _ = OamShape::find_smallest_containing(GeneralResolution::new(0, 10));
        assert_eq!(
            OamShape::find_smallest_containing(GeneralResolution::new(90, 10)),
            None
        );
    }
}
