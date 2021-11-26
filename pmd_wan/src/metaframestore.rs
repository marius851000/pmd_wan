use crate::{MetaFrame, MetaFrameGroup, Resolution, WanError};
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug)]
pub struct MetaFrameStore {
    pub meta_frames: Vec<MetaFrame>,
    pub meta_frame_groups: Vec<MetaFrameGroup>,
}

impl MetaFrameStore {
    // assume that the pointer is already well positionned
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        nb_meta_frame: u64,
    ) -> Result<MetaFrameStore, WanError> {
        let mut meta_frames = Vec::new();
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
            meta_frame_groups.push(MetaFrameGroup::new_from_bytes(file, &mut meta_frames)?);
        }
        Ok(MetaFrameStore {
            meta_frames,
            meta_frame_groups,
        })
    }

    pub fn find_resolution_and_pal_idx_image(
        &self,
        image_id: u32,
    ) -> Result<(Option<Resolution<u8>>, u16), WanError> {
        for actual_image in &self.meta_frames {
            if actual_image.image_index == image_id as usize {
                return Ok((actual_image.resolution, actual_image.pal_idx));
            };
        }
        Err(WanError::InvalidResolution)
    }

    pub fn write<F: Write + Seek>(
        file: &mut F,
        meta_frame_store: &MetaFrameStore,
    ) -> Result<Vec<u32>, WanError> {
        let nb_meta_frame = meta_frame_store.meta_frame_groups.len();
        let mut meta_frame_references = vec![];

        for l in 0..nb_meta_frame {
            meta_frame_references.push(file.seek(SeekFrom::Current(0))? as u32);
            MetaFrameGroup::write(
                file,
                &meta_frame_store.meta_frame_groups[l],
                &meta_frame_store.meta_frames,
            )?;
        }

        Ok(meta_frame_references)
    }
}
