/// Size of an [`Image`] (computed from [`MetaFrame`])
#[derive(Debug, Copy, Clone)]
pub struct Resolution<T> {
    pub x: T,
    pub y: T,
}

impl Resolution<u8> {
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
}

impl Resolution<u8> {
    pub fn get_indice(self) -> Option<(u8, u8)> {
        Some(match (self.y, self.x) {
            (8, 8) => (0, 0),
            (16, 16) => (0, 1),
            (32, 32) => (0, 2),
            (64, 64) => (0, 3),
            (16, 8) => (1, 0),
            (8, 16) => (2, 0),
            (32, 8) => (1, 1),
            (8, 32) => (2, 1),
            (32, 16) => (1, 2),
            (16, 32) => (2, 2),
            (64, 32) => (1, 3),
            (32, 64) => (2, 3),
            _ => return None,
        })
    }
}
