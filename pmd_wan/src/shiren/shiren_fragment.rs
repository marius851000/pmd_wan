use std::io::Read;

use byteorder::{ReadBytesExt, LE};

use crate::{get_bit_u16, OamShape, WanError};

#[derive(Debug)]
pub struct ShirenFragment {
    // None for 0xFFFF, Some otherwise
    pub fragment_bytes_id: Option<u16>,
    pub unk1: u16,
    pub unk3: Option<u16>,
    pub unk4: u16,
    pub is_h_flip: bool,
    pub unk5: u16,
    pub oam_shape: OamShape,
    pub offset_x: i16,
    pub offset_y: i16,
}

impl ShirenFragment {
    /// Return None if it’s a frame end marker
    pub fn new<T: Read>(reader: &mut T) -> Result<Option<Self>, WanError> {
        let fragment_bytes_id = reader.read_u16::<LE>()?;
        let unk1 = reader.read_u16::<LE>()?;
        if fragment_bytes_id == 0xFFFF && unk1 == 0xFFFF {
            return Ok(None);
        }
        let unk3;
        if unk1 & 0x0080 == 0 || fragment_bytes_id == 0xFFFF {
            unk3 = Some(reader.read_u16::<LE>()?);
        } else {
            unk3 = None;
        }
        let unk4 = reader.read_u16::<LE>()?;
        let unk5 = reader.read_u16::<LE>()?;

        let is_h_flip = get_bit_u16(unk4, 3).unwrap();
        let _some_transformed_unk = unk4 & 0xe00 >> 9;
        //TODO: there’s probably a vertical flip too
        let size_indice = if unk3.is_some() {
            (unk4 >> 14) as u8
        } else {
            2
        };
        let shape_indice = unk3.map(|x| (x >> 14) as u8).unwrap_or(0);
        let oam_shape = if let Some(oam_shape) = OamShape::new(shape_indice, size_indice) {
            oam_shape
        } else {
            return Err(WanError::InvalidResolutionIndice(shape_indice, size_indice));
        };

        let offset_x = (unk4 & 0x1FF) as i16 - 256;
        let offset_y = unk3.map(|x| ((x & 0xFF) as i8) as i16).unwrap_or(0);
        Ok(Some(Self {
            fragment_bytes_id: if fragment_bytes_id == 0xFFFF {
                None
            } else {
                Some(fragment_bytes_id)
            },
            unk1,
            unk3,
            unk4,
            is_h_flip,
            unk5,
            oam_shape,
            offset_x,
            offset_y,
        }))
    }
}
