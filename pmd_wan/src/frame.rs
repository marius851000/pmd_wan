use anyhow::{bail, Context};

use crate::{Fragment, WanError};
use std::io::{Read, Write};

/// A single frame of animation
#[derive(Debug, PartialEq, Eq, Clone, Default)]
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
            trace!("its data: {:?}", fragments[fragments.len() - 1]);
            if is_last {
                break;
            }
        }
        Ok(Frame { fragments })
    }

    pub fn write<F: Write>(&self, file: &mut F) -> anyhow::Result<()> {
        let mut previous_image: Option<usize> = None;
        if self.fragments.is_empty() {
            bail!("A frame has no fragment, which can’t be encoded.");
        }
        for (fragment_nb, fragment) in self.fragments.iter().enumerate() {
            fragment
                .write(
                    file,
                    previous_image,
                    fragment_nb + 1 == self.fragments.len(),
                )
                .with_context(move || format!("Can't write the fragment {:?}", fragment))?;
            previous_image = Some(fragment.image_index);
        }
        Ok(())
    }

    pub fn generate_size_to_allocate_for_max_metaframe(&self) -> u32 {
        self.fragments
            .iter()
            .map(|x| {
                x.image_alloc_counter as u32 + x.resolution.chunk_to_allocate_for_metaframe() as u32
            })
            .max()
            .unwrap_or(0)
    }
}
