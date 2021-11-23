#[macro_use]
extern crate log;

pub mod wanimage;
pub use wanimage::WanImage;

mod wanerror;
pub use wanerror::WanError;

mod metaframegroup;
pub use metaframegroup::MetaFrameGroup;

mod metaframe;
pub use metaframe::MetaFrame;

mod resolution;
pub use resolution::Resolution;

mod metaframestore;
pub use metaframestore::MetaFrameStore;

mod spritetype;
pub use spritetype::SpriteType;

mod imagebytes;
pub use crate::imagebytes::{ImageBytes, ImageBytesToImageError};

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

fn get_bit_u16(byte: u16, id: u16) -> Option<bool> {
    if id < 8 {
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

pub struct Coordinate {
    x: u32,
    y: u32,
}
