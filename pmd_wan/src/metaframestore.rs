use crate::{MetaFrameGroup, WanError};
use anyhow::Context;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug, Default)]
pub struct MetaFrameStore {
    pub meta_frame_groups: Vec<MetaFrameGroup>,
}

impl MetaFrameStore {
    // assume that the pointer is already well positionned
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        nb_meta_frame: u64,
    ) -> Result<MetaFrameStore, WanError> {
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
            meta_frame_groups.push(MetaFrameGroup::new_from_bytes(file)?);
        }
        Ok(MetaFrameStore { meta_frame_groups })
    }

    pub fn write<F: Write + Seek>(&self, file: &mut F) -> anyhow::Result<Vec<u32>> {
        let mut meta_frame_references = vec![];

        for meta_frame_group in &self.meta_frame_groups {
            meta_frame_references.push(file.seek(SeekFrom::Current(0))? as u32);
            meta_frame_group.write(file).with_context(move || {
                format!("can't write the meta frame group {:?}", meta_frame_group)
            })?;
        }

        Ok(meta_frame_references)
    }
}
