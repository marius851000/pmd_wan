#[macro_use]
extern crate log;

#[cfg(test)]
pub mod tests;

pub mod wan_image;
pub use wan_image::WanImage;

mod wan_error;
pub use wan_error::WanError;

mod frame;
pub use frame::Frame;

mod fragment;
pub use fragment::Fragment;

mod oam_shape;
pub use oam_shape::OamShape;

mod frame_store;
pub use frame_store::FrameStore;

mod sprite_type;
pub use sprite_type::SpriteType;

mod fragment_bytes;
pub use crate::fragment_bytes::{
    decode_fragment_pixels, encode_fragment_pixels, DecodeFragmentBytesError, FragmentBytes,
    FragmentBytesToImageError,
};

mod palette;
pub use palette::Palette;

mod fragment_bytes_store;
pub use fragment_bytes_store::FragmentBytesStore;

mod animation_frame;
pub use animation_frame::AnimationFrame;

mod animation_store;
pub use animation_store::AnimationStore;

mod animation;
pub use animation::Animation;

mod fragment_bytes_compression;
pub use fragment_bytes_compression::*;

mod fragment_flip;
pub use fragment_flip::{
    FragmentFlip, FragmentFlipError, FLIP_BOTH, FLIP_HORIZONTAL, FLIP_STANDARD, FLIP_VERTICAL,
};

mod fragment_finder;
pub use fragment_finder::{
    find_fragments_in_images, pad_seven_pixel, FragmentFinderData, FragmentFinderError,
};

mod image_to_wan;
pub use image_to_wan::insert_frame_in_wanimage;

mod image_buffer;
pub use image_buffer::ImageBuffer;

pub mod image_tool;

mod multi_images_to_wan;
pub use multi_images_to_wan::create_wan_from_multiple_images;

mod normalized_bytes;
pub use normalized_bytes::{NormalizedBytes, VariableNormalizedBytes};

mod frame_offset;
pub use frame_offset::FrameOffset;

use binwrite::WriterOption;
pub fn get_opt_le() -> WriterOption {
    binwrite::writer_option_new!(endian: binwrite::Endian::Little)
}

#[cfg(feature = "shiren_experimental")]
pub mod shiren;

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

    pub fn can_contain(&self, other: Self) -> bool {
        self.x >= other.x && self.y >= other.y
    }
}

/// A utility function that get the byte at the position "id" from "bytes".
/// From the left.
/// If the id is >= 16, then it return None, otherwise return Some.
pub fn get_bit_u16(bytes: u16, id: u16) -> Option<bool> {
    if id < 16 {
        Some((bytes >> (15 - id) << 15) >= 1)
    } else {
        None
    }
}

fn wan_read_raw_4<F: std::io::Read>(file: &mut F) -> Result<[u8; 4], WanError> {
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests_lib {
    use crate::GeneralResolution;

    #[test]
    pub fn test_can_contain() {
        assert!(GeneralResolution::new(64, 64).can_contain(GeneralResolution::new(32, 64)));
        assert!(!GeneralResolution::new(8, 8).can_contain(GeneralResolution::new(30, 4)));
        assert!(!GeneralResolution::new(0, 0).can_contain(GeneralResolution::new(10, 10)));
    }
}
