use crate::{AnimationFrame, WanError};
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
}

impl Animation {
    pub fn new<F: Read>(file: &mut F) -> Result<Animation, WanError> {
        let mut frames = Vec::new();
        loop {
            let current_frame = AnimationFrame::new(file)?;
            if current_frame.is_null() {
                break;
            }
            frames.push(current_frame);
        }
        Ok(Animation { frames })
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write<F: Write>(file: &mut F, animation: &Animation) -> Result<(), WanError> {
        for frame in &animation.frames {
            AnimationFrame::write(file, frame)?;
        }
        AnimationFrame::write_null(file)?;
        Ok(())
    }
}
