use crate::get_bit_u16;
use crate::{Resolution, WanError};
use binwrite::{BinWrite, Endian, WriterOption, writer_option_new};
use byteorder::{BigEndian, LE, ReadBytesExt};
use std::io::{Read, Write};

//TODO: add a to_image to convert to an [`image`] (and make it an optional dependancy btw)
#[derive(Debug, PartialEq, Eq)]
pub struct MetaFrame {
    pub unk1: u16,
    pub unk2: u16,
    pub unk3: bool,
    pub unk4: bool,
    pub unk5: bool,
    pub image_index: usize,
    pub offset_y: i8,
    pub offset_x: i16,
    pub is_last: bool,
    pub v_flip: bool,
    pub h_flip: bool,
    pub is_mosaic: bool,
    pub pal_idx: u16,
    pub resolution: Option<Resolution<u8>>,
}

impl MetaFrame {
    pub fn new_from_bytes<F: Read>(
        file: &mut F,
        previous_image: Option<usize>,
    ) -> Result<MetaFrame, WanError> {
        trace!("parsing a meta-frame");
        let image_index = match file.read_i16::<LE>()? {
            -1 => match previous_image {
                None => return Err(WanError::ImageIDPointBackButFirstImage),
                Some(value) => {
                    value
                }
            },
            x => {
                if x >= 0 {
                    x as usize
                } else {
                    return Err(WanError::MetaFrameLessThanLessOne(x));
                }
            }
        };

        let unk1 = file.read_u16::<LE>()?;

        // they are quite strangely encoded (the fact I should read as little-endian the 2 byte correctly reading them)

        // bit in ppmdu tool are right to left !!!
        let offset_y_data = file.read_u16::<LE>()?;
        let size_indice_y = ((0xC000 & offset_y_data) >> (8 + 6)) as u8;
        let is_mosaic = get_bit_u16(offset_y_data, 3).unwrap(); //no panic: always return if indice less than 16
        let unk3 = get_bit_u16(offset_y_data, 7).unwrap();
        let unk4 = get_bit_u16(offset_y_data, 6).unwrap();
        let offset_y = (offset_y_data & 0x00FF) as i8; //range: 0-255 (integer)

        let offset_x_data = file.read_u16::<LE>()?;
        let size_indice_x = ((0xC000 & offset_x_data) >> (8 + 6)) as u8;
        let v_flip = get_bit_u16(offset_x_data, 2).unwrap(); //as no panic as before
        let h_flip = get_bit_u16(offset_x_data, 3).unwrap();
        let is_last = get_bit_u16(offset_x_data, 4).unwrap();
        let unk5 = get_bit_u16(offset_x_data, 5).unwrap();
        let offset_x = (offset_x_data & 0x01FF) as i16 - 256; //range: 0-511
        
        let unk2 = file.read_u16::<LE>()?;
        let pal_idx = ((0xF000 & unk2) >> 12) as u16;

        Ok(MetaFrame {
            unk1,
            unk2,
            unk3,
            unk4,
            unk5,
            image_index,
            offset_x,
            offset_y,
            is_last,
            v_flip,
            h_flip,
            is_mosaic,
            pal_idx,
            resolution: Resolution::from_indice(size_indice_x, size_indice_y),
        })
    }

    pub fn is_last(&self) -> bool {
        self.is_last
    }

    pub fn write<F: Write>(
        &self,
        file: &mut F,
        previous_image: Option<usize>,
    ) -> Result<(), WanError> {
        let image_index: i16 = match previous_image {
            None => self.image_index as i16,
            Some(value) => {
                if self.image_index == value {
                    -1
                } else {
                    self.image_index as i16
                }
            }
        };

        (image_index, self.unk1).write(file)?;

        let (size_indice_x, size_indice_y) = match self.resolution {
            Some(value) => match value.get_indice() {
                Some(value) => value,
                //HACK:
                None => panic!(),
            },
            //HACK:
            None => panic!(),
        };

        let offset_y_data: u16 = ((size_indice_y as u16) << (8 + 6))
            + if self.is_mosaic { 1 << (8 + 4) } else { 0 }
            + ((self.unk4 as u16) << (8 + 1))
            + ((self.unk3 as u16) << 8)
            + ((self.offset_y as u16) & 0x00FF);

        let offset_x_data: u16 = ((size_indice_x as u16) << (8 + 6))
            + ((self.v_flip as u16) << (8 + 5))
            + ((self.h_flip as u16) << (8 + 4))
            + ((self.is_last as u16) << (8 + 3))
            + ((self.unk5 as u16) << (8 + 2))
            + (((self.offset_x + 256) as u16) & 0x01FF);

        (offset_y_data, offset_x_data, self.unk2).write(file)?;

        Ok(())
    }
}
