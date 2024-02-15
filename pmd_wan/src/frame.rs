use anyhow::{bail, Context};

use crate::{Fragment, FrameOffset, WanError};
use std::io::{Read, Write};

/// A single frame of animation
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Frame {
    pub fragments: Vec<Fragment>,
    /// While this is stored in a separate part of the file, they are mapped to a Frame
    pub frame_offset: Option<FrameOffset>,
}

impl Frame {
    pub fn new_from_bytes<F: Read>(file: &mut F) -> Result<Frame, WanError> {
        let mut fragments = Vec::new();
        let mut previous_fragment_bytes = None;
        loop {
            let (fragment, is_last) = Fragment::new_from_bytes(file, previous_fragment_bytes)?;
            previous_fragment_bytes = Some(fragment.fragment_bytes_index);
            fragments.push(fragment);
            trace!("its data: {:?}", fragments[fragments.len() - 1]);
            if is_last {
                break;
            }
        }
        Ok(Frame {
            fragments,
            frame_offset: None,
        })
    }

    /// Returns: size to allocate for the fragments of this frame
    pub fn write<F: Write>(&self, file: &mut F) -> anyhow::Result<u16> {
        let mut previous_fragment_bytes: Option<usize> = None;
        if self.fragments.is_empty() {
            bail!("A frame has no fragment, which can’t be encoded.");
        }
        let mut fragment_alloc_counter = 0;
        for (fragment_nb, fragment) in self.fragments.iter().enumerate() {
            fragment
                .write(
                    file,
                    previous_fragment_bytes,
                    fragment_nb + 1 == self.fragments.len(),
                    fragment_alloc_counter,
                )
                .with_context(move || format!("Can't write the fragment {:?}", fragment))?;
            fragment_alloc_counter += fragment.resolution.chunk_to_allocate_for_fragment();
            previous_fragment_bytes = Some(fragment.fragment_bytes_index);
        }
        Ok(fragment_alloc_counter)
    }

    /// Returns: size to allocate for the fragments of this frame
    pub fn compute_fragment_alloc_counter(&self) -> u16 {
        self.fragments
            .iter()
            .map(|f| f.resolution.chunk_to_allocate_for_fragment())
            .sum()
    }
}
