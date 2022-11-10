use crate::{Frame, WanError};
use anyhow::Context;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug, Default)]
pub struct FrameStore {
    pub frames: Vec<Frame>,
}

impl FrameStore {
    // assume that the pointer is already well positionned
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        nb_meta_frame: u64,
    ) -> Result<FrameStore, WanError> {
        let mut meta_frame_groups = Vec::new();
        let mut last_pointer = None;

        let mut meta_frame_reference: Vec<u64> = Vec::new();
        for _ in 0..nb_meta_frame {
            let actual_ptr = file.read_u32::<LE>()? as u64;
            //some check
            match last_pointer {
                None => last_pointer = Some(actual_ptr),
                Some(value) => {
                    if actual_ptr
                        .checked_sub(value)
                        .ok_or(WanError::InvalidOffset)?
                        % 10
                        != 0
                    {
                        return Err(WanError::InvalidOffset);
                    }
                }
            };
            meta_frame_reference.push(actual_ptr);
        }

        for meta_frame_id in 0..nb_meta_frame {
            trace!(
                "parsing meta-frame n°{} (at offset {})",
                meta_frame_id,
                meta_frame_reference[meta_frame_id as usize]
            );
            file.seek(SeekFrom::Start(
                meta_frame_reference[meta_frame_id as usize],
            ))?;
            meta_frame_groups.push(Frame::new_from_bytes(file)?);
        }
        Ok(FrameStore {
            frames: meta_frame_groups,
        })
    }

    //Return: (List of offset to the encoded frames, max allocation size for a frame)
    pub fn write<F: Write + Seek>(&self, file: &mut F) -> anyhow::Result<(Vec<u32>, u16)> {
        let mut frame_references = vec![];
        let mut size_to_allocate = 0;

        for frame in &self.frames {
            frame_references.push(file.seek(SeekFrom::Current(0))? as u32);
            let local_size_to_allocate = frame
                .write(file)
                .with_context(move || format!("can't write the meta frame group {:?}", frame))?;
            size_to_allocate = size_to_allocate.max(local_size_to_allocate);
        }

        Ok((frame_references, size_to_allocate))
    }
}
