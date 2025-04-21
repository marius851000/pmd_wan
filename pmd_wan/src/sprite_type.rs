use crate::CompressionMethod;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpriteType {
    PropsUI,
    Chara,
    Unk2,
    Unk3,
}

impl SpriteType {
    pub fn get_id(self) -> u8 {
        match self {
            SpriteType::PropsUI => 0,
            SpriteType::Chara => 1,
            SpriteType::Unk2 => 2,
            SpriteType::Unk3 => 3,
        }
    }

    pub fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(SpriteType::PropsUI),
            1 => Some(SpriteType::Chara),
            2 => Some(SpriteType::Unk2),
            3 => Some(SpriteType::Unk3),
            _ => None,
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
