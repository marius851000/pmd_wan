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
        while let Some(fragment) = ShirenFragment::new(reader)? {
            fragments.push(fragment);
        }
        Ok(Self { fragments })
    }
}
