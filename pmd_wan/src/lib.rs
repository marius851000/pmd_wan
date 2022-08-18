#[macro_use]
extern crate log;

#[cfg(test)]
pub mod tests;

pub mod wanimage;
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

mod image_to_wan;
pub use image_to_wan::insert_fragment_in_wanimage;

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
