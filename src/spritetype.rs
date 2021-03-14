#[derive(Debug, PartialEq)]
pub enum SpriteType {
    PropsUI,
    Chara,
    Unknown,
}

impl SpriteType {
    #[allow(dead_code)]
    fn get_id(&self) -> u8 {
        match self {
            SpriteType::PropsUI => 0,
            SpriteType::Chara => 1,
            SpriteType::Unknown => 3,
        }
    }
}
