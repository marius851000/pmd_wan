use std::io::Read;

use byteorder::{LE, ReadBytesExt};

use crate::WanError;

#[derive(Debug)]
pub struct ShirenFragment {
    pub fragment_bytes_id: u16,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: Option<u16>,
    pub unk4: u16,
    pub unk5: u16,
    pub size_indice_x: u8, //TODO: for debugging
    pub size_indice_y: u8, //TODO: idem
}

impl ShirenFragment {
    pub fn new<T: Read>(reader: &mut T) -> Result<Self, WanError> {
        let fragment_bytes_id = reader.read_u16::<LE>()?;
        let unk1 = reader.read_u8()?;
        let unk2 = reader.read_u8()?;
        let unk3;
        if unk1 & 0x80 == 0 {
            unk3 = Some(reader.read_u16::<LE>()?);
        } else {
            unk3 = None;
        }
        let unk4 = reader.read_u16::<LE>()?;
        let unk5 = reader.read_u16::<LE>()?;
        let size_indice_y = ((0xC000 & unk4) >> (8 + 6)) as u8;
        let size_indice_x = ((0xC000 & unk5) >> (8 + 6)) as u8;
        //TODO
        Ok(Self {
            fragment_bytes_id,
            unk1,
            unk2,
            unk3,
            unk4,
            unk5,
            size_indice_x,
            size_indice_y
        })
    }

    pub fn is_end_marker(&self) -> bool {
        return self.fragment_bytes_id == 0xFFFF; //TODO: a more detailed check.
    }
}