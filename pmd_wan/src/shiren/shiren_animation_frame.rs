use byteorder::{LittleEndian, ReadBytesExt};

use crate::WanError;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ShirenAnimationFrame {
    pub unk1: u16,
    pub maybe_frame: u16,
    pub unk2: [u8; 8],
}

impl ShirenAnimationFrame {
    pub fn new<T: Read>(reader: &mut T) -> Result<Self, WanError> {
        let unk1 = reader.read_u16::<LittleEndian>()?;
        let maybe_frame = reader.read_u16::<LittleEndian>()?;
        let mut unk2 = [0; 8];
        reader.read_exact(&mut unk2)?;
        return Ok(Self {
            unk1,
            maybe_frame,
            unk2,
        });
    }

    pub fn is_end_marker(&self) -> bool {
        self.unk1 == 0 && self.maybe_frame == 0 //TODO: the other value should probably be 0 too
    }
}
