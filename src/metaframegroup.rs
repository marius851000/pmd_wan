use crate::{MetaFrame, WanError};
use std::io::Read;

#[derive(Debug)]
pub struct MetaFrameGroup {
    pub meta_frames_id: Vec<usize>,
}

impl MetaFrameGroup {
    pub fn new_from_bytes<F: Read>(
        file: &mut F,
        meta_frames: &mut Vec<MetaFrame>,
    ) -> Result<MetaFrameGroup, WanError> {
        let mut meta_frames_id = Vec::new();
        let mut previous_image = None;
        loop {
            meta_frames_id.push(meta_frames.len()); // We refer to the metaframe we will put here
            let meta_frame = MetaFrame::new_from_bytes(file, previous_image)?;
            previous_image = Some(meta_frame.image_index);
            meta_frames.push(meta_frame);
            trace!("it's data: {:?}", meta_frames[meta_frames.len() - 1]);
            if meta_frames[meta_frames.len() - 1].is_last() {
                break;
            }
        }
        Ok(MetaFrameGroup { meta_frames_id })
    }

    /*fn write<F: Write>(
        file: &mut F,
        meta_frame_group: &MetaFrameGroup,
        meta_frames: &[MetaFrame],
    ) -> Result<(), WanError> {
        let mut previous_image: Option<usize> = None;
        for l in 0..meta_frame_group.meta_frames_id.len() {
            let meta_frames_id = meta_frame_group.meta_frames_id[l];
            let meta_frame_to_write = &meta_frames[meta_frames_id];
            MetaFrame::write(file, meta_frame_to_write, previous_image)?;
            previous_image = Some(l);
        }
        Ok(())
    }*/
}
