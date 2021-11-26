use crate::WanError;
use binwrite::BinWrite;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Write};

/// A single frame of an [`crate::Animation`]
#[derive(Debug, PartialEq, Clone, Eq)]
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

    pub fn write<F: Write>(file: &mut F, frame: &AnimationFrame) -> Result<(), WanError> {
        (
            frame.duration,
            frame.flag,
            frame.frame_id,
            frame.offset_x,
            frame.offset_y,
            frame.shadow_offset_x,
            frame.shadow_offset_y,
        )
            .write(file)?;

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
    }
}
