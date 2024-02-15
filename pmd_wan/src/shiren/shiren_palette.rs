use std::io::{Read, Seek};

use binread::BinReaderExt;

use crate::WanError;

#[derive(Debug)]

pub struct ShirenPalette {
    /// Orders of color in pixel are R, G, B, A (0x80 for fully visible, 0 for transparent)
    pub colors: [[u8; 4]; 192],
}

impl ShirenPalette {
    pub fn new<T: Read + Seek>(reader: &mut T) -> Result<Self, WanError> {
        let mut colors = [[0; 4]; 192];
        for color_key in 0..colors.len() {
            colors[color_key] = reader.read_le()?;
            if color_key % 16 == 0 {
                colors[color_key] = [0, 0, 0, 0];
            }
        }
        Ok(Self { colors })
    }
}
