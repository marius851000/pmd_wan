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
        nb_frames: u64,
    ) -> Result<FrameStore, WanError> {
        let mut frames = Vec::new();
        let mut last_pointer = None;

        let mut fragment_reference: Vec<u64> = Vec::new();
        for _ in 0..nb_frames {
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
            fragment_reference.push(actual_ptr);
        }

        for frame_id in 0..nb_frames {
            trace!(
                "parsing frame n°{} (at offset {})",
                frame_id,
                fragment_reference[frame_id as usize]
            );
            file.seek(SeekFrom::Start(
                fragment_reference[frame_id as usize],
            ))?;
            frames.push(Frame::new_from_bytes(file)?);
        }
        Ok(FrameStore {
            frames,
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
                .with_context(move || format!("can't write the frame group {:?}", frame))?;
            size_to_allocate = size_to_allocate.max(local_size_to_allocate);
        }

        Ok((frame_references, size_to_allocate))
    }

    /// Returns: max allocation size for a frame
    pub fn compute_fragment_alloc_counter(&self) -> u16 {
        self.frames.iter().map(|f| f.compute_fragment_alloc_counter()).max().unwrap_or(0)
    }
}
