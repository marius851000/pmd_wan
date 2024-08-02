/// The pair of valid size and related indice. The first value is the size (x, y), the second is the indice pair.
static VALID_SIZE_AND_INDICE: [([u8; 2], [u8; 2]); 12] = [
    ([8, 8], [0, 0]),
    ([16, 16], [1, 0]),
    ([32, 32], [2, 0]),
    ([64, 64], [3, 0]),
    ([16, 8], [0, 1]),
    ([8, 16], [0, 2]),
    ([32, 8], [1, 1]),
    ([8, 32], [1, 2]),
    ([32, 16], [2, 1]),
    ([16, 32], [2, 2]),
    ([64, 32], [3, 1]),
    ([32, 64], [3, 2]),
];

/// Size of an [`crate::ImageBytes`] (computed from [`crate::Fragment`])
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FragmentResolution {
    pub x: u8,
    pub y: u8,
}

impl FragmentResolution {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub fn from_indice(indice_x: u8, indice_y: u8) -> Option<Self> {
        for entry in &VALID_SIZE_AND_INDICE {
            if indice_x == entry.1[0] && indice_y == entry.1[1] {
                return Some(Self {
                    x: entry.0[0],
                    y: entry.0[1],
                });
            };
        }
        None
    }

    /// Return the indices for the given image resolution.
    /// The first one is the X indices, and the second one, the Y indices
    pub fn get_indice(self) -> Option<(u8, u8)> {
        for entry in &VALID_SIZE_AND_INDICE {
            if entry.0[0] == self.x && entry.0[1] == self.y {
                return Some((entry.1[0], entry.1[1]));
            };
        }
        None
    }

    pub fn chunk_to_allocate_for_fragment(&self) -> u16 {        
        let blocks = ((self.x as u16) * (self.y as u16) / 256) as u16;
        if blocks >= 1 {blocks} else {1}
    }

    pub fn can_contain(self, other: Self) -> bool {
        self.x >= other.x && self.y >= other.y
    }

    /// Return the smallest resolution (in term of allocation) that can contain the target resolution.
    ///
    /// If there are multiple posible resolution with the same number of size to allocate, returnt the one with the lesser amount of pixel. If there are still multiple remaining resolution, return any possible one (implementation detail: they aren't random).
    pub fn find_smaller_containing(
        target_resolution: FragmentResolution,
    ) -> Option<FragmentResolution> {
        let mut optimal_result: Option<(u16, u16, FragmentResolution)> = None; // first u16 is number of chunk to allocate for the frame, second u16 is the number of pixel, third is the optimal resolution right now
        for entry in &VALID_SIZE_AND_INDICE {
            let resolution_entry = FragmentResolution::new(entry.0[0], entry.0[1]);
            if resolution_entry.can_contain(target_resolution) {
                let chunk_to_allocate_entry = resolution_entry.chunk_to_allocate_for_fragment();
                let pixel_nb_entry = (resolution_entry.x as u16) * (resolution_entry.y as u16);
                if let Some((chunk_to_allocate_optimal, pixel_nb_optimal, _)) = &optimal_result {
                    if *chunk_to_allocate_optimal > chunk_to_allocate_entry
                        || (*chunk_to_allocate_optimal == chunk_to_allocate_entry
                            && *pixel_nb_optimal > pixel_nb_entry)
                    {
                        optimal_result =
                            Some((chunk_to_allocate_entry, pixel_nb_entry, resolution_entry));
                    }
                } else {
                    optimal_result =
                        Some((chunk_to_allocate_entry, pixel_nb_entry, resolution_entry));
                };
            }
        }
        optimal_result.map(|x| x.2)
    }

    /// Return the number of pixel an image with this resolution can contain (x×y)
    pub fn nb_pixels(self) -> u16 {
        self.x as u16 * self.y as u16
    }
}

//TODO: check by decoding every images too
#[cfg(test)]
mod tests {
    use crate::FragmentResolution;
    #[test]
    fn test_resolution_chunk_allocation() {
        for ((input_x, input_y), expected_output) in &[((32, 32), 4), ((32, 8), 1), ((64, 64), 16)]
        {
            let resolution = FragmentResolution {
                x: *input_x,
                y: *input_y,
            };
            let got = resolution.chunk_to_allocate_for_fragment();
            if got != *expected_output {
                panic!(
                    "The resolution {:?} return the allocation number {}, but {} were expected",
                    resolution, got, expected_output
                );
            }
        }
    }

    #[test]
    pub fn test_can_contain() {
        assert!(FragmentResolution::new(64, 64).can_contain(FragmentResolution::new(32, 64)));
        assert!(!FragmentResolution::new(8, 8).can_contain(FragmentResolution::new(30, 4)));
        assert!(!FragmentResolution::new(0, 0).can_contain(FragmentResolution::new(10, 10)));
    }

    #[test]
    pub fn test_find_smaller() {
        assert_eq!(
            FragmentResolution::find_smaller_containing(FragmentResolution::new(6, 3)).unwrap(),
            FragmentResolution::new(8, 8)
        );
        assert_eq!(
            FragmentResolution::find_smaller_containing(FragmentResolution::new(64, 10)).unwrap(),
            FragmentResolution::new(64, 32)
        );
        let _ = FragmentResolution::find_smaller_containing(FragmentResolution::new(0, 10));
        assert_eq!(
            FragmentResolution::find_smaller_containing(FragmentResolution::new(90, 10)),
            None
        );
    }
}
