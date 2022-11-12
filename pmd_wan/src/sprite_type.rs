use crate::CompressionMethod;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpriteType {
    PropsUI,
    Chara,
    Unknown,
}

impl SpriteType {
    pub fn get_id(self) -> u8 {
        match self {
            SpriteType::PropsUI => 0,
            SpriteType::Chara => 1,
            SpriteType::Unknown => 3,
        }
    }

    pub fn default_compression_method(self) -> CompressionMethod {
        if self == SpriteType::Chara {
            CompressionMethod::CompressionMethodOriginal
        } else {
            CompressionMethod::NoCompression
        }
    }
}
