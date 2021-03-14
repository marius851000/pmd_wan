use crate::{wan_read_raw_4, WanError};
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

pub struct Palette {
    pub palette: Vec<(u8, u8, u8, u8)>,
}

impl Palette {
    pub fn new_from_bytes<F: Read + Seek>(file: &mut F) -> Result<Palette, WanError> {
        let mut palette = Vec::new();
        let pointer_palette_start = file.read_u32::<LE>()? as u64;
        file.read_u16::<LE>()?;
        let nb_color = file.read_u16::<LE>()?;
        wan_read_raw_4(file)?;
        trace!(
            "palette_start: {}, nb_color: {}",
            pointer_palette_start,
            nb_color
        );
        if file.read_u32::<LE>()? != 0 {
            return Err(WanError::PaletteDontEndWithZero);
        };
        file.seek(SeekFrom::Start(pointer_palette_start))?;
        for _ in 0..nb_color {
            let red = file.read_u8()?;
            let green = file.read_u8()?;
            let blue = file.read_u8()?;
            let alpha = file.read_u8()?;
            palette.push((red, green, blue, alpha));
        }
        Ok(Palette { palette })
    }

    pub fn get(&self, id: usize) -> Result<(u8, u8, u8, u8), WanError> {
        if id >= self.palette.len() {
            return Err(WanError::PaletteOOB);
        };
        Ok(self.palette[id])
    }

    #[allow(dead_code)]
    pub fn color_id(&self, target_color: (u8, u8, u8, u8)) -> Result<usize, WanError> {
        for color_id in 0..self.palette.len() {
            if self.palette[color_id] == target_color {
                return Ok(color_id);
            }
        }
        error!("impossible to find the palette {:?}", target_color);
        error!("tried :");
        for color in &self.palette {
            error!("{:?}", color);
        }
        Err(WanError::CantFindColorInPalette)
    }

    /*fn write<F: Write + Seek>(&self, file: &mut F) -> Result<u64, WanError> {
        let palette_start_offset = file.seek(SeekFrom::Current(0))?;
        for color in &self.palette {
            wan_write_u8(file, color.0)?;
            wan_write_u8(file, color.1)?;
            wan_write_u8(file, color.2)?;
            wan_write_u8(file, color.3)?;
        }

        let palette_header_offset = file.seek(SeekFrom::Current(0))?;
        wan_write_u32(file, palette_start_offset as u32)?;
        wan_write_u16(file, 0)?; //unk
        wan_write_u16(file, 16)?; //TODO: assume the picture is 4bit
        wan_write_u32(file, 0xFF << 16)?; //unk
        wan_write_u32(file, 0)?; //magic

        Ok(palette_header_offset)
    }*/
}
