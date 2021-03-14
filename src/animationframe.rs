use std::io::{Read};
use byteorder::{LE, ReadBytesExt};
use crate::{WanError};

#[derive(Debug, PartialEq, Clone)]
pub struct AnimationFrame {
    pub duration: u8,
    pub flag: u8,
    pub frame_id: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub shadow_offset_x: i16,
    pub shadow_offset_y: i16,
}

impl AnimationFrame {
    pub fn new<F: Read>(file: &mut F) -> Result<AnimationFrame, WanError> {
        let duration = file.read_u8()?;
        let flag = file.read_u8()?;
        let frame_id = file.read_u16::<LE>()?;
        let offset_x = file.read_i16::<LE>()?;
        let offset_y = file.read_i16::<LE>()?;
        let shadow_offset_x = file.read_i16::<LE>()?;
        let shadow_offset_y = file.read_i16::<LE>()?;
        Ok(AnimationFrame {
            duration,
            flag,
            frame_id,
            offset_x,
            offset_y,
            shadow_offset_x,
            shadow_offset_y,
        })
    }

    pub fn is_null(&self) -> bool {
        self.duration == 0 && self.frame_id == 0
    }

    /*pub fn write<F: Write>(file: &mut F, frame: &AnimationFrame) -> Result<(), WanError> {
        wan_write_u8(file, frame.duration)?;
        wan_write_u8(file, frame.flag)?;
        wan_write_u16(file, frame.frame_id)?;
        wan_write_i16(file, frame.offset_x)?;
        wan_write_i16(file, frame.offset_y)?;
        wan_write_i16(file, frame.shadow_offset_x)?;
        wan_write_i16(file, frame.shadow_offset_y)?;
        Ok(())
    }

    pub fn write_null<F: Write>(file: &mut F) -> Result<(), WanError> {
        AnimationFrame::write(
            file,
            &AnimationFrame {
                duration: 0,
                flag: 0,
                frame_id: 0,
                offset_x: 0,
                offset_y: 0,
                shadow_offset_x: 0,
                shadow_offset_y: 0,
            },
        )
    }*/
}
