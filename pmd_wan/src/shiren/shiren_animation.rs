use super::ShirenAnimationFrame;
use crate::WanError;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ShirenAnimation {
    pub frames: Vec<ShirenAnimationFrame>,
}

impl ShirenAnimation {
    pub fn new<T: Read>(reader: &mut T) -> Result<Self, WanError> {
        let mut frames = Vec::new();
        loop {
            let frame = ShirenAnimationFrame::new(reader)?;
            if frame.is_end_marker() {
                break;
            }
            frames.push(frame);
        }

        Ok(Self { frames })
    }
}
