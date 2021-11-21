#[derive(Debug, PartialEq)]
pub enum SpriteType {
    PropsUI,
    Chara,
    Unknown,
}

impl SpriteType {
    pub fn get_id(&self) -> u8 {
        match self {
            SpriteType::PropsUI => 0,
            SpriteType::Chara => 1,
            SpriteType::Unknown => 3,
        }
    }
}
