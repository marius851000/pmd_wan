#[derive(Debug, Copy, Clone)]
pub struct Resolution<T> {
    pub x: T,
    pub y: T,
}

/*impl Resolution<u8> {
    pub fn get_indice(self) -> Option<(u16, u16)> {
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
}*/
