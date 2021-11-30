/// Size of an [`Image`] (computed from [`MetaFrame`])
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub x: u8,
    pub y: u8,
}

impl Resolution {
    pub fn from_indice(indice_x: u8, indice_y: u8) -> Option<Self> {
        match (indice_x, indice_y) {
            (0, 0) => Some(Self { x: 8, y: 8 }),
            (1, 0) => Some(Self { x: 16, y: 16 }),
            (2, 0) => Some(Self { x: 32, y: 32 }),
            (3, 0) => Some(Self { x: 64, y: 64 }),
            (0, 1) => Some(Self { x: 16, y: 8 }),
            (0, 2) => Some(Self { x: 8, y: 16 }),
            (1, 1) => Some(Self { x: 32, y: 8 }),
            (1, 2) => Some(Self { x: 8, y: 32 }),
            (2, 1) => Some(Self { x: 32, y: 16 }),
            (2, 2) => Some(Self { x: 16, y: 32 }),
            (3, 1) => Some(Self { x: 64, y: 32 }),
            (3, 2) => Some(Self { x: 32, y: 64 }),
            _ => None,
        }
    }

    /// Return the indices for the given image resolution.
    /// The first one is the X indices, and the second one, the Y indices
    pub fn get_indice(self) -> Option<(u8, u8)> {
        Some(match (self.x, self.y) {
            (8, 8) => (0, 0),
            (16, 16) => (1, 0),
            (32, 32) => (2, 0),
            (64, 64) => (3, 0),
            (16, 8) => (0, 1),
            (8, 16) => (0, 2),
            (32, 8) => (1, 1),
            (8, 32) => (1, 2),
            (32, 16) => (2, 1),
            (16, 32) => (2, 2),
            (64, 32) => (3, 1),
            (32, 64) => (3, 2),
            _ => return None,
        })
    }
}
