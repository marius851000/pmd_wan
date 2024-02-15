use std::io::Read;

use crate::WanError;

use super::ShirenFragment;

#[derive(Debug)]
pub struct ShirenFrame {
    pub fragments: Vec<ShirenFragment>,
}

impl ShirenFrame {
    pub fn new<T: Read>(reader: &mut T) -> Result<Self, WanError> {
        let mut fragments = Vec::new();
        loop {
            let fragment = ShirenFragment::new(reader)?;
            trace!("read fragment {:?}", fragment);
            if fragment.is_end_marker() {
                break;
            }
            fragments.push(fragment);
        }
        Ok(Self { fragments })
    }
}
