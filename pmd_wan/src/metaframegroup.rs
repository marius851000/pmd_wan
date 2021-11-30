use anyhow::Context;

use crate::{MetaFrame, WanError};
use std::io::{Read, Write};

/// A single frame of animation
#[derive(Debug, PartialEq, Eq)]
pub struct MetaFrameGroup {
    pub meta_frames: Vec<MetaFrame>,
}

impl MetaFrameGroup {
    pub fn new_from_bytes<F: Read>(file: &mut F) -> Result<MetaFrameGroup, WanError> {
        let mut meta_frames = Vec::new();
        let mut previous_image = None;
        loop {
            let (meta_frame, is_last) = MetaFrame::new_from_bytes(file, previous_image)?;
            previous_image = Some(meta_frame.image_index);
            meta_frames.push(meta_frame);
            trace!("it's data: {:?}", meta_frames[meta_frames.len() - 1]);
            if is_last {
                break;
            }
        }
        Ok(MetaFrameGroup { meta_frames })
    }

    pub fn write<F: Write>(&self, file: &mut F) -> anyhow::Result<()> {
        let mut previous_image: Option<usize> = None;
        for (mf_nb, meta_frame) in self.meta_frames.iter().enumerate() {
            meta_frame
                .write(file, previous_image, mf_nb + 1 == self.meta_frames.len())
                .with_context(move || format!("Can't write the meta_frame {:?}", meta_frame))?;
            previous_image = Some(meta_frame.image_index);
        }
        Ok(())
    }
}
