use crate::get_bit_u16;
use crate::FragmentFlip;
use crate::OamShape;
use crate::WanError;
use anyhow::bail;
use byteorder::WriteBytesExt;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Write};

/// A [`Fragment`] may reference an [`crate::FragmentBytes`], that will form a single (or all if small enought) part of an [`crate::Frame`]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fragment {
    pub unk1: u16,
    /// Two value with unknown property in the offset y data.
    /// most of the time, the unk3 is equal to offset_y < 0, and unk4 the inverse (will be automatically computed if set to None)
    /// otherwise the two boolean in the tuple will be used
    pub unk3_4: Option<(bool, bool)>,
    pub unk5: bool, // maybe is "invert palette color"
    pub fragment_bytes_index: usize,
    pub offset_y: i8,
    pub offset_x: i16,
    pub flip: FragmentFlip,
    pub is_mosaic: bool,
    pub pal_idx: u16,
    pub resolution: OamShape,
}

impl Fragment {
    /// parse a metaframe from the file.
    /// The second value is whether the "is_last" bit has been set to true, meaning it's the last Fragment from the Frame
    pub fn new_from_bytes<F: Read>(
        file: &mut F,
        previous_fragment_bytes: Option<usize>,
    ) -> Result<(Fragment, bool), WanError> {
        trace!("parsing a fragment");
        let fragment_bytes_index = match file.read_i16::<LE>()? {
            -1 => match previous_fragment_bytes {
                None => return Err(WanError::FragmentBytesIDPointBackButFirstFragment),
                Some(value) => value,
            },
            x => {
                if x >= 0 {
                    x as usize
                } else {
                    return Err(WanError::FragmentLessThanLessOne(x));
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
        let offset_y = (offset_y_data & 0x00FF) as i8; //range: 0-255 (integer) //TODO: shouldn’t that be u8?

        #[allow(clippy::collapsible_else_if)]
        let unk3_4 = if offset_y < 0 {
            if unk3 && !unk4 {
                None
            } else {
                Some((unk3, unk4))
            }
        } else {
            if !unk3 && unk4 {
                None
            } else {
                Some((unk3, unk4))
            }
        };

        let offset_x_data = file.read_u16::<LE>()?;
        let size_indice_x = ((0xC000 & offset_x_data) >> (8 + 6)) as u8;
        let v_flip = get_bit_u16(offset_x_data, 2).unwrap(); //as no panic as before
        let h_flip = get_bit_u16(offset_x_data, 3).unwrap();
        let flip = FragmentFlip::from_bools(v_flip, h_flip);
        let is_last = get_bit_u16(offset_x_data, 4).unwrap();
        let unk5 = get_bit_u16(offset_x_data, 5).unwrap();
        let offset_x = (offset_x_data & 0x01FF) as i16 - 256; //range: 0-511

        let alloc_and_palette = file.read_u16::<LE>()?;
        let pal_idx = (0xF000 & alloc_and_palette) >> 12;

        Ok((
            Fragment {
                unk1,
                unk3_4,
                unk5,
                fragment_bytes_index,
                offset_x,
                offset_y,
                flip,
                is_mosaic,
                pal_idx,
                resolution: match OamShape::new(size_indice_y, size_indice_x) {
                    Some(r) => r,
                    None => {
                        return Err(WanError::InvalidResolutionIndice(
                            size_indice_x,
                            size_indice_y,
                        ))
                    }
                },
            },
            is_last,
        ))
    }

    pub fn write<F: Write>(
        &self,
        file: &mut F,
        previous_fragment_bytes: Option<usize>,
        is_last: bool,
        fragment_alloc_counter: u16,
    ) -> anyhow::Result<()> {
        //TODO: use try_into, or maybe even directly i16
        let fragment_bytes_index: i16 = match previous_fragment_bytes {
            None => self.fragment_bytes_index as i16,
            Some(value) => {
                if self.fragment_bytes_index == value {
                    -1
                } else {
                    self.fragment_bytes_index as i16
                }
            }
        };

        file.write_i16::<LE>(fragment_bytes_index)?;
        file.write_u16::<LE>(self.unk1)?;

        let (unk3, unk4) = match self.unk3_4 {
            Some((unk3, unk4)) => (unk3, unk4),
            None => {
                let unk3 = self.offset_y < 0;
                (unk3, !unk3)
            }
        };

        let offset_y_data: u16 = ((self.resolution.shape_indice() as u16) << (8 + 6))
            + if self.is_mosaic { 1 << (8 + 4) } else { 0 }
            + ((unk4 as u16) << (8 + 1))
            + ((unk3 as u16) << 8)
            + ((self.offset_y as u16) & 0x00FF);

        let written_offset_x = self.offset_x + 256;
        if written_offset_x >= 0x200 {
            bail!(
                "The x coordinate of this metaframe is more than 255 (it is {})",
                self.offset_x
            );
        }
        if written_offset_x < 0 {
            bail!(
                "The x coordinate of this metaframe is less than 256 (it is {})",
                self.offset_x
            );
        }

        let (v_flip, h_flip) = self.flip.to_bools();

        let offset_x_data: u16 = ((self.resolution.size_indice() as u16) << (8 + 6))
            + ((v_flip as u16) << (8 + 5))
            + ((h_flip as u16) << (8 + 4))
            + ((is_last as u16) << (8 + 3))
            + ((self.unk5 as u16) << (8 + 2))
            + (((written_offset_x) as u16) & 0x01FF);

        file.write_u16::<LE>(offset_y_data)?;
        file.write_u16::<LE>(offset_x_data)?;
        file.write_u16::<LE>(
            ((self.pal_idx & 0xF) << 12) + 0x0C00 + (fragment_alloc_counter & 0x3FF),
        )?;

        Ok(())
    }
}
