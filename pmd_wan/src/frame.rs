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
            let (fragment, is_last) = Fragment::new_from_bytes(file, previous_image)?;
            previous_image = Some(fragment.image_bytes_index);
            fragments.push(fragment);
            trace!("its data: {:?}", fragments[fragments.len() - 1]);
            if is_last {
                break;
            }
        }
        Ok(Frame { fragments })
    }

    /// Returns: size to allocate for this image
    pub fn write<F: Write>(&self, file: &mut F) -> anyhow::Result<u16> {
        let mut previous_image_bytes: Option<usize> = None;
        if self.fragments.is_empty() {
            bail!("A frame has no fragment, which can’t be encoded.");
        }
        let mut image_alloc_counter = 0;
        for (fragment_nb, fragment) in self.fragments.iter().enumerate() {
            fragment
                .write(
                    file,
                    previous_image_bytes,
                    fragment_nb + 1 == self.fragments.len(),
                    image_alloc_counter,
                )
                .with_context(move || format!("Can't write the fragment {:?}", fragment))?;
            image_alloc_counter += fragment.resolution.chunk_to_allocate_for_fragment();
            previous_image_bytes = Some(fragment.image_bytes_index);
        }
        Ok(image_alloc_counter)
    }
}
