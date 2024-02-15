use arr_macro::arr;
use byteorder::{LittleEndian, ReadBytesExt};

use super::ShirenAnimation;
use crate::WanError;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct ShirenAnimationStore {
    pub animations: Vec<[ShirenAnimation; 8]>,
}

impl ShirenAnimationStore {
    // Assume the file originally point at the animation group pointer list
    pub fn new<T: Read + Seek>(
        reader: &mut T,
        animation_group_amount: u32,
    ) -> Result<Self, WanError> {
        let mut animation_pointer_pointers = Vec::with_capacity(animation_group_amount as usize);
        for _ in 0..animation_group_amount {
            animation_pointer_pointers.push(reader.read_u32::<LittleEndian>()?);
        }
        let mut animations = Vec::with_capacity(animation_group_amount as usize);
        for animation_pointer_pointer in animation_pointer_pointers {
            reader.seek(SeekFrom::Start(animation_pointer_pointer.into()))?;
            let mut animation_pointers = [0; 8];
            for pointer_count in 0..8 {
                animation_pointers[pointer_count] = reader.read_u32::<LittleEndian>()?;
            }

            let mut counter = 0;
            let animation_group = arr![{
                reader.seek(SeekFrom::Start(animation_pointers[counter].into()))?;
                counter += 1;
                ShirenAnimation::new(reader)?
            }; 8];
            animations.push(animation_group);
        }
        Ok(Self { animations })
    }
}
