use byteorder::{LittleEndian, ReadBytesExt};

use crate::WanError;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ShirenAnimationFrame {
    pub frame_duration: u8,
    pub unk3: u8,
    pub frame_id: u16,
    pub unk2: [u8; 8],
}

impl ShirenAnimationFrame {
    pub fn new<T: Read>(reader: &mut T) -> Result<Self, WanError> {
        let frame_duration = reader.read_u8()?;
        let unk3 = reader.read_u8()?;
        let frame_id = reader.read_u16::<LittleEndian>()?;
        let mut unk2 = [0; 8];
        reader.read_exact(&mut unk2)?;
        return Ok(Self {
            frame_duration,
            unk3,
            frame_id,
            unk2,
        });
    }

    pub fn is_end_marker(&self) -> bool {
        self.frame_duration == 0 && self.frame_id == 0 //TODO: the other value should probably be 0 too
    }
}
