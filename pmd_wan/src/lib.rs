#[macro_use]
extern crate log;

#[cfg(test)]
pub mod tests;

pub mod wanimage;
use binwrite::WriterOption;
pub use wanimage::WanImage;

mod wanerror;
pub use wanerror::WanError;

mod frame;
pub use frame::Frame;

mod fragment;
pub use fragment::Fragment;

mod fragment_resolution;
pub use fragment_resolution::FragmentResolution;

mod frame_store;
pub use frame_store::FrameStore;

mod spritetype;
pub use spritetype::SpriteType;

mod imagebytes;
pub use crate::imagebytes::{
    decode_fragment_pixels, encode_fragment_pixels, DecodeImageError, ImageBytes,
    ImageBytesToImageError,
};

mod palette;
pub use palette::Palette;

mod imagestore;
pub use imagestore::ImageStore;

mod animationframe;
pub use animationframe::AnimationFrame;

mod animstore;
pub use animstore::AnimStore;

mod animation;
pub use animation::Animation;

mod imagecompression;
pub use imagecompression::*;

mod fragment_flip;
pub use fragment_flip::{FragmentFlip, FragmentFlipError};

mod fragment_finder;
pub use fragment_finder::{
    find_fragments_in_images, pad_seven_pixel, FragmentFinderData, FragmentFinderError,
};

mod image_to_wan;
pub use image_to_wan::insert_fragment_in_wanimage;

pub mod image_tool;

mod multi_images_to_wan;
pub use multi_images_to_wan::create_wan_from_multiple_images;

mod normalized_bytes;
pub use normalized_bytes::{NormalizedBytes, VariableNormalizedBytes};

pub fn get_opt_le() -> WriterOption {
    binwrite::writer_option_new!(endian: binwrite::Endian::Little)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct GeneralResolution {
    pub x: u32,
    pub y: u32,
}

impl GeneralResolution {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn nb_pixels(&self) -> u64 {
        (self.x as u64) * (self.y as u64)
    }
}

fn get_bit_u16(byte: u16, id: u16) -> Option<bool> {
    if id < 16 {
        Some((byte >> (15 - id) << 15) >= 1)
    } else {
        None
    }
}

fn wan_read_raw_4<F: std::io::Read>(file: &mut F) -> Result<[u8; 4], WanError> {
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
