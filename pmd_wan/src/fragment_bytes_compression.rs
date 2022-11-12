use std::io::{Seek, SeekFrom, Write};

use byteorder::WriteBytesExt;

use crate::{fragment_bytes::FragmentBytesAssemblyEntry, FragmentBytes, WanError};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CompressionMethod {
    /// The compression used to compress creatures in base game
    CompressionMethodOriginal,
    /// No compression, used for other sprites in base game
    NoCompression,
    /* /// An original optimised compression algorithm (TODO: I think it is unfinished, need testing, or maybe just fuzzing)
    CompressionMethodOptimised {
        multiple_of_value: usize,
        min_transparent_to_compress: usize,
    },*/
}

impl CompressionMethod {
    pub fn compress<F: Write + Seek>(
        &self,
        fragment_bytes: &FragmentBytes,
        pixel_list: &[u8],
        file: &mut F,
    ) -> Result<Vec<FragmentBytesAssemblyEntry>, WanError> {
        let compression = if pixel_list.len() % 64 != 0 {
            CompressionMethod::NoCompression
        } else {
            self.clone()
        };

        if pixel_list.is_empty() {
            return Err(WanError::EmptyFragmentBytes);
        }

        let mut assembly_table: Vec<FragmentBytesAssemblyEntry> = vec![];

        match compression {
            Self::CompressionMethodOriginal => {
                enum ActualEntry {
                    Null(u32, u32),      //lenght (pixel), z_index
                    Some(u64, u32, u32), // initial_offset, lenght (pixel), z_index
                }

                impl ActualEntry {
                    fn new(is_all_black: bool, start_offset: u64, z_index: u32) -> ActualEntry {
                        if is_all_black {
                            ActualEntry::Null(64, z_index)
                        } else {
                            ActualEntry::Some(start_offset, 64, z_index)
                        }
                    }

                    fn to_assembly(&self) -> FragmentBytesAssemblyEntry {
                        match self {
                            ActualEntry::Null(lenght, z_index) => FragmentBytesAssemblyEntry {
                                pixel_src: 0,
                                pixel_amount: *lenght,
                                byte_amount: (*lenght / 2) as u16, //NOTE: lenght is always <= than 64x64
                                _z_index: *z_index,
                            },
                            ActualEntry::Some(initial_offset, lenght, z_index) => {
                                FragmentBytesAssemblyEntry {
                                    pixel_src: *initial_offset,
                                    pixel_amount: *lenght,
                                    byte_amount: (*lenght / 2) as u16,
                                    _z_index: *z_index,
                                }
                            }
                        }
                    }

                    fn advance(&self, lenght: u32) -> ActualEntry {
                        match self {
                            ActualEntry::Null(l, z) => ActualEntry::Null(*l + lenght, *z),
                            ActualEntry::Some(offset, l, z) => {
                                ActualEntry::Some(*offset, *l + lenght, *z)
                            }
                        }
                    }
                }

                let mut actual_entry: Option<ActualEntry> = None;

                for (loop_nb, _chunk) in pixel_list.chunks_exact(64).enumerate() {
                    let mut this_area = vec![];
                    let mut is_all_black = true;
                    for l in 0..64 {
                        let actual_pixel = pixel_list[(loop_nb * 64 + l) as usize];
                        this_area.push(actual_pixel);
                        if actual_pixel != 0 {
                            is_all_black = false;
                        };
                    }

                    let pos_before_area = file.seek(SeekFrom::Current(0))?;
                    if !is_all_black {
                        for byte_id in 0..32 {
                            file.write_u8(
                                (this_area[byte_id * 2] << 4) + this_area[byte_id * 2 + 1],
                            )?;
                        }
                    }

                    let need_to_create_new_entry = match &actual_entry {
                        Some(ActualEntry::Null(_, _)) => !is_all_black,
                        Some(ActualEntry::Some(_, _, _)) => is_all_black,
                        None => true,
                    };

                    actual_entry = if need_to_create_new_entry {
                        if let Some(entry) = actual_entry {
                            assembly_table.push(entry.to_assembly())
                        }

                        Some(ActualEntry::new(
                            is_all_black,
                            pos_before_area,
                            fragment_bytes.z_index,
                        ))
                    } else {
                        // no panic : need_to_create_new_entry is false if actual_entry is none
                        Some(actual_entry.unwrap().advance(64))
                    }
                }
                assembly_table.push(actual_entry.unwrap().to_assembly())
            }
            /*Self::CompressionMethodOptimised {
                multiple_of_value,
                min_transparent_to_compress,
            } => {
                let mut number_of_byte_to_include: u16 = 0;
                let mut byte_include_start = file.seek(SeekFrom::Current(0))?;

                let mut pixel_id = 0;
                loop {
                    debug_assert!(pixel_id % 2 == 0);
                    let mut should_create_new_transparent_entry = false;

                    if (pixel_id % multiple_of_value == 0)
                        && (pixel_id + min_transparent_to_compress < pixel_list.len())
                    {
                        let mut encontered_non_transparent = false;
                        for l in 0..min_transparent_to_compress {
                            if pixel_list[pixel_id + l] != 0 {
                                encontered_non_transparent = true;
                                break;
                            };
                        }
                        if !encontered_non_transparent {
                            should_create_new_transparent_entry = true;
                        };
                    };

                    if should_create_new_transparent_entry {
                        //push the actual content
                        if number_of_byte_to_include > 0 {
                            assembly_table.push(ImageAssemblyEntry {
                                pixel_src: byte_include_start,
                                pixel_amount: number_of_byte_to_include as u32 * 2,
                                byte_amount: number_of_byte_to_include,
                                _z_index: image.z_index,
                            });
                            number_of_byte_to_include = 0;
                            byte_include_start = file.seek(SeekFrom::Current(0))?;
                        };
                        //create new entry for transparent stuff
                        //count the number of transparent tile
                        let mut transparent_tile_nb: u32 = 0; //TODO: somehow guarantee it never gets bigger than (16^2)*2
                        loop {
                            if pixel_id >= pixel_list.len() {
                                break;
                            };
                            if pixel_list[pixel_id] == 0 {
                                transparent_tile_nb += 1;
                                pixel_id += 1;
                            } else {
                                break;
                            };
                        }
                        if pixel_id % multiple_of_value != 0 {
                            transparent_tile_nb -= (pixel_id % multiple_of_value) as u32;
                            pixel_id -= pixel_id % multiple_of_value;
                        };
                        assembly_table.push(ImageAssemblyEntry {
                            pixel_src: 0,
                            pixel_amount: transparent_tile_nb,
                            byte_amount: (transparent_tile_nb / 2) as u16, //TODO: take care of the tileset lenght
                            _z_index: image.z_index,
                        });

                        continue;
                    };

                    if pixel_id >= pixel_list.len() {
                        break;
                    };
                    debug_assert!(pixel_list[pixel_id] < 16);
                    debug_assert!(pixel_list[pixel_id + 1] < 16);
                    file.write_u8(((pixel_list[pixel_id] << 4) + pixel_list[pixel_id + 1]) as u8)?;
                    pixel_id += 2;
                    number_of_byte_to_include += 1;
                }
                if number_of_byte_to_include > 0 {
                    assembly_table.push(ImageAssemblyEntry {
                        pixel_src: byte_include_start,
                        pixel_amount: number_of_byte_to_include as u32 * 2,
                        byte_amount: number_of_byte_to_include,
                        _z_index: image.z_index,
                    });
                };
            }*/
            Self::NoCompression => {
                let mut byte_len = 0;
                let start_offset = file.seek(SeekFrom::Current(0))?;
                for pixels in pixel_list.chunks_exact(2) {
                    file.write_u8((pixels[0] << 4) + pixels[1])?;
                    byte_len += 1;
                }
                assembly_table.push(FragmentBytesAssemblyEntry {
                    pixel_src: start_offset,
                    pixel_amount: byte_len * 2,
                    byte_amount: byte_len as u16,
                    _z_index: fragment_bytes.z_index,
                })
            }
        };
        Ok(assembly_table)
    }
}
