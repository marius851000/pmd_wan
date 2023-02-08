use std::io::{Seek, Read};

use byteorder::{ReadBytesExt, LE};

use crate::WanError;

use super::ShirenFrame;

#[derive(Debug)]
pub struct ShirenFrameStore {
    pub frames: Vec<ShirenFrame>
}

impl ShirenFrameStore {
    pub fn new<T: Read + Seek>(reader: &mut T, nb_fragments: u32) -> Result<Self, WanError> {
        let mut pointers = Vec::new();
        for _ in 0..nb_fragments {
            pointers.push(reader.read_u32::<LE>()?);
        }
        let mut frames = Vec::new();
        for pointer in pointers {
            trace!("Reading frames at {}", pointer);
            reader.seek(std::io::SeekFrom::Start(pointer as u64))?;
            let frame = ShirenFrame::new(reader)?;
            frames.push(frame);
        };
        Ok(Self {
            frames
        })
    }
}