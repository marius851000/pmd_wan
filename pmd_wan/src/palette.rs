use crate::{wan_read_raw_4, WanError};
use binwrite::BinWrite;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug, Default)]
/// A palette, composed of group of 16 color when the first is transparent. Colors are RGBA.
pub struct Palette {
    pub palette: Vec<[u8; 4]>,
}

impl Palette {
    /// load the Palette. Assume the cursor it located at the palette header
    pub fn new_from_bytes<F: Read + Seek>(file: &mut F) -> Result<Palette, WanError> {
        let mut palette = Vec::new();
        let pointer_palette_start = file.read_u32::<LE>()? as u64;
        trace!("start of palette : {}", pointer_palette_start);

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
            palette.push([red, green, blue, alpha]);
        }
        Ok(Palette { palette })
    }

    /// Return the rgba color for the given color id and palette id.
    /// Return [`Option::None`] if the color doesn't exist.
    /// Note that the alpha range from 0 to 128 normally, thought this is not an hard guarantee.
    pub fn get(&self, id: u8, palette_id: u16) -> Option<[u8; 4]> {
        let id = (id as usize).saturating_add(palette_id as usize * 16);
        if id >= self.palette.len() {
            return None;
        };
        Some(self.palette[id])
    }

    #[allow(dead_code)]
    pub fn color_id(&self, target_color: [u8; 4], palette_id: u16) -> Result<usize, WanError> {
        for color_id in (palette_id as usize) * 16..(palette_id as usize) * 16 + self.palette.len()
        {
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

    pub fn write<F: Write + Seek>(&self, file: &mut F) -> Result<u64, WanError> {
        let start_offset = file.seek(SeekFrom::Current(0))?;
        for color in &self.palette {
            color.write(file)?;
        }

        let header_offset = file.seek(SeekFrom::Current(0))?;
        (
            start_offset as u32,
            0u16, //unk
            self.palette.len() as u16,
            (0xFF << 16) as u32, //unk
            0u32,                //magic
        )
            .write(file)?;

        Ok(header_offset)
    }
}
