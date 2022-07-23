use anyhow::Context;

use crate::{Fragment, WanError};
use std::io::{Read, Write};

/// A single frame of animation
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub fragments: Vec<Fragment>,
}

impl Frame {
    pub fn new_from_bytes<F: Read>(file: &mut F) -> Result<Frame, WanError> {
        let mut fragments = Vec::new();
        let mut previous_image = None;
        loop {
            let (meta_frame, is_last) = Fragment::new_from_bytes(file, previous_image)?;
            previous_image = Some(meta_frame.image_index);
            fragments.push(meta_frame);
            trace!("it's data: {:?}", fragments[fragments.len() - 1]);
            if is_last {
                break;
            }
        }
        Ok(Frame { fragments })
    }

    pub fn write<F: Write>(&self, file: &mut F) -> anyhow::Result<()> {
        let mut previous_image: Option<usize> = None;
        for (mf_nb, meta_frame) in self.fragments.iter().enumerate() {
            meta_frame
                .write(file, previous_image, mf_nb + 1 == self.fragments.len())
                .with_context(move || format!("Can't write the meta_frame {:?}", meta_frame))?;
            previous_image = Some(meta_frame.image_index);
        }
        Ok(())
    }
}
