use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

use crate::{resolution::Resolution, Palette, WanError};

#[derive(Debug)]
pub struct ImageAssemblyEntry {
    pixel_src: u64,
    pixel_amount: u64,
    byte_amount: u64,
    _z_index: u32,
}

impl ImageAssemblyEntry {
    fn new_from_bytes<F: Read>(file: &mut F) -> Result<ImageAssemblyEntry, WanError> {
        let pixel_src = file.read_u32::<LE>()? as u64;
        let byte_amount = file.read_u16::<LE>()? as u64;
        let pixel_amount = byte_amount * 2;
        file.read_u16::<LE>()?;
        let z_index = file.read_u32::<LE>()?;
        Ok(ImageAssemblyEntry {
            pixel_src,
            pixel_amount,
            byte_amount,
            _z_index: z_index,
        })
    }

    fn is_null(&self) -> bool {
        self.pixel_amount == 0 && self.pixel_src == 0
    }

    /*fn write<F: Write>(&self, file: &mut F) -> Result<(), WanError> {
        wan_write_u32(file, self.pixel_src as u32)?;
        wan_write_u16(file, self.byte_amount as u16)?;
        wan_write_u16(file, 0)?;
        wan_write_u32(file, self._z_index)?;
        Ok(())
    }*/
}

/// an helper struct, that permit to know where to place the next pixel
struct ImgPixelPointer {
    xsize: u32,
    //ysize: u32,
    true_column: u32,
    true_line: u32,
    line: u32,
    column: u32,
}

impl ImgPixelPointer {
    fn new(resx: u32, _resy: u32) -> ImgPixelPointer {
        ImgPixelPointer {
            xsize: resx,
            //ysize: resy,
            true_column: 0,
            true_line: 0,
            line: 0,
            column: 0,
        }
    }

    fn next(&mut self) -> Resolution<u32> {
        let tile_width = 8;
        let tile_height = 8;

        let mut x = self.true_column * 8 + self.column;
        match self.column % 2 {
            0 => x += 1,
            1 => x -= 1,
            _ => panic!(),
        };
        let y = self.true_line * 8 + self.line;
        self.column += 1;
        if self.column >= tile_width {
            self.column = 0;
            self.line += 1;
            if self.line >= tile_height {
                self.line = 0;
                self.true_column += 1;
                if self.true_column >= self.xsize / tile_width {
                    self.true_column = 0;
                    self.true_line += 1;
                }
            }
        }
        Resolution::<u32> { x, y }
    }
}

pub struct Image {
    pub img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub z_index: u32,
}

impl Image {
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        resolution: Resolution<u8>,
        pal_idx: u16,
        palette: &Palette,
    ) -> Result<Image, WanError> {
        let mut img_asm_table = Vec::new();
        let mut image_size = 0;

        let mut last_pointer = None; //for check
        loop {
            let asm_entry = ImageAssemblyEntry::new_from_bytes(file)?;
            image_size += asm_entry.pixel_amount;
            if asm_entry.is_null() {
                break;
            } else {
                trace!(
                    "part amount: {}, point to: {}",
                    asm_entry.pixel_amount,
                    asm_entry.pixel_src
                );
                if asm_entry.pixel_src != 0 {
                    match last_pointer {
                        None => last_pointer = Some(asm_entry.pixel_src + asm_entry.byte_amount),
                        Some(value) => {
                            if value == asm_entry.pixel_src {
                                last_pointer = Some(asm_entry.byte_amount + value);
                            } else {
                                return Err(WanError::IncoherentPointerToImagePart);
                            }
                        }
                    }
                };
                img_asm_table.push(asm_entry);
            }
        }
        trace!(
            "the resolution of this image is ({}, {})",
            resolution.x,
            resolution.y
        );
        trace!(
            "the image contain {} assembly entry, with an image size of {}.",
            img_asm_table.len(),
            image_size
        );

        // transform to image on the fly
        let mut img = image::ImageBuffer::new(resolution.x as u32, resolution.y as u32);

        let mut img_pixel_pointer = ImgPixelPointer::new(resolution.x as u32, resolution.y as u32);
        let mut z_index = None;

        let test_out_of_bound = |out_resolution: &Resolution<u32>| {
            let x_res = resolution.x as u32;
            let y_res = resolution.y as u32;
            if out_resolution.x >= x_res || out_resolution.y >= y_res {
                Err(WanError::SpriteTooSmall)
            } else {
                Ok(())
            }
        };

        for entry in &img_asm_table {
            if entry.pixel_src == 0 {
                for _ in 0..entry.pixel_amount {
                    let pixel_pos = img_pixel_pointer.next();
                    test_out_of_bound(&pixel_pos)?;
                    let pixel = img.get_pixel_mut(pixel_pos.x, pixel_pos.y);
                    *pixel = image::Rgba([0, 0, 0, 0]);
                }
            } else {
                file.seek(SeekFrom::Start(entry.pixel_src))?;
                let mut actual_byte = 0;
                for loop_id in 0..entry.pixel_amount {
                    let color_id = if loop_id % 2 == 0 {
                        actual_byte = file.read_u8()?;
                        actual_byte >> 4
                    } else {
                        (actual_byte << 4) >> 4
                    };
                    let pixel_pos = img_pixel_pointer.next();
                    test_out_of_bound(&pixel_pos)?;
                    let pixel = img.get_pixel_mut(pixel_pos.x, pixel_pos.y);
                    if color_id == 0 {
                        *pixel = image::Rgba([0, 0, 0, 0]);
                    } else {
                        let color = palette.get((pal_idx * 16 + color_id as u16) as usize)?;
                        let color = [color.0, color.1, color.2, 255];
                        *pixel = image::Rgba(color);
                    };
                }
            };
            // check that all part of the image have the same z index
            if let Some(index) = z_index {
                if index != entry._z_index {
                    return Err(WanError::NonConstantIndexInImage);
                };
            };
            z_index = Some(entry._z_index);
        }

        let z_index = match z_index {
            Some(value) => value,
            None => return Err(WanError::NoZIndex),
        };

        Ok(Image { img, z_index })
    }

    /*fn write<F: Write + Seek>(
        &self,
        file: &mut F,
        palette: &Palette,
    ) -> Result<(u64, Vec<u64>), WanError> {
        let _resolution = (self.img.width(), self.img.height());
        // generate the pixel list
        let group_resolution = (self.img.width() / 8, self.img.height() / 8);

        let mut pixel_list: Vec<u8> = vec![]; //a value = a pixel (acording to the tileset). None is fully transparent

        for y_group in 0..group_resolution.1 {
            for x_group in 0..group_resolution.0 {
                for in_group_y in 0..8 {
                    for in_group_x in &[1, 0, 3, 2, 5, 4, 7, 6] {
                        let real_x_pixel = x_group * 8 + in_group_x;
                        let real_y_pixel = y_group * 8 + in_group_y;
                        let real_color = self.img.get_pixel(real_x_pixel, real_y_pixel);
                        let real_color_tuple = (
                            real_color[0],
                            real_color[1],
                            real_color[2],
                            match real_color[3] {
                                255 => 128,
                                0 => 0,
                                _ => return Err(WanError::ImpossibleAlphaLevel),
                            },
                        );
                        pixel_list.push(if real_color_tuple == (0, 0, 0, 0) {
                            0
                        } else {
                            palette.color_id(real_color_tuple)? as u8
                        });
                    }
                }
            }
        }
        // those value allow to have the same output of the original file
        //TODO: make the value change if we don't need to have the same output
        let min_transparent_to_compress = 32; //TODO: take care of the palette len
        let multiple_of_value = 2;
        let use_legacy_compression = true; //set to false to use a better compression algo

        let mut assembly_table: Vec<ImageAssemblyEntry> = vec![];

        if !use_legacy_compression {
            let mut number_of_byte_to_include = 0;
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
                            pixel_amount: number_of_byte_to_include * 2,
                            byte_amount: number_of_byte_to_include,
                            _z_index: self.z_index,
                        });
                        number_of_byte_to_include = 0;
                        byte_include_start = file.seek(SeekFrom::Current(0))?;
                    };
                    //create new entry for transparent stuff
                    //count the number of transparent tile
                    let mut transparent_tile_nb = 0;
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
                        transparent_tile_nb -= pixel_id % multiple_of_value;
                        pixel_id -= pixel_id % multiple_of_value;
                    };
                    assembly_table.push(ImageAssemblyEntry {
                        pixel_src: 0,
                        pixel_amount: transparent_tile_nb as u64,
                        byte_amount: (transparent_tile_nb as u64) / 2, //TODO: take care of the tileset lenght
                        _z_index: self.z_index,
                    });

                    continue;
                };

                if pixel_id >= pixel_list.len() {
                    break;
                };
                debug_assert!(pixel_list[pixel_id] < 16);
                debug_assert!(pixel_list[pixel_id + 1] < 16);
                wan_write_u8(file, (pixel_list[pixel_id] << 4) + pixel_list[pixel_id + 1])?;
                pixel_id += 2;
                number_of_byte_to_include += 1;
            }
            if number_of_byte_to_include > 0 {
                assembly_table.push(ImageAssemblyEntry {
                    pixel_src: byte_include_start,
                    pixel_amount: number_of_byte_to_include * 2,
                    byte_amount: number_of_byte_to_include,
                    _z_index: self.z_index,
                });
            };
        } else {
            enum ActualEntry {
                Null(u64, u32),      //lenght (pixel), z_index
                Some(u64, u64, u32), // initial_offset, lenght (pixel), z_index
            }

            impl ActualEntry {
                fn new(is_all_black: bool, start_offset: u64, z_index: u32) -> ActualEntry {
                    if is_all_black {
                        ActualEntry::Null(64, z_index)
                    } else {
                        ActualEntry::Some(start_offset, 64, z_index)
                    }
                }

                fn to_assembly(&self) -> ImageAssemblyEntry {
                    match self {
                        ActualEntry::Null(lenght, z_index) => ImageAssemblyEntry {
                            pixel_src: 0,
                            pixel_amount: *lenght,
                            byte_amount: *lenght / 2,
                            _z_index: *z_index,
                        },
                        ActualEntry::Some(initial_offset, lenght, z_index) => ImageAssemblyEntry {
                            pixel_src: *initial_offset,
                            pixel_amount: *lenght,
                            byte_amount: *lenght / 2,
                            _z_index: *z_index,
                        },
                    }
                }

                fn advance(&self, lenght: u64) -> ActualEntry {
                    match self {
                        ActualEntry::Null(l, z) => ActualEntry::Null(*l + lenght, *z),
                        ActualEntry::Some(offset, l, z) => {
                            ActualEntry::Some(*offset, *l + lenght, *z)
                        }
                    }
                }
            }

            let mut actual_entry: Option<ActualEntry> = None;

            for loop_nb in 0..(group_resolution.0 * group_resolution.1) {
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
                        wan_write_u8(
                            file,
                            (this_area[byte_id * 2] << 4) + this_area[byte_id * 2 + 1],
                        )?;
                    }
                }

                let need_to_create_new_entry = if actual_entry.is_none() {
                    true
                } else {
                    match &actual_entry {
                        Some(ActualEntry::Null(_, _)) => !is_all_black,
                        Some(ActualEntry::Some(_, _, _)) => is_all_black,
                        _ => panic!(),
                    }
                };

                if need_to_create_new_entry {
                    if let Some(entry) = actual_entry {
                        assembly_table.push(entry.to_assembly())
                    }

                    actual_entry = Some(ActualEntry::new(
                        is_all_black,
                        pos_before_area,
                        self.z_index,
                    ));
                } else {
                    //TODO:
                    actual_entry = Some(actual_entry.unwrap().advance(64));
                }
            }
            assembly_table.push(actual_entry.unwrap().to_assembly())
        }
        //insert empty entry
        assembly_table.push(ImageAssemblyEntry {
            pixel_src: 0,
            pixel_amount: 0,
            byte_amount: 0,
            _z_index: 0,
        });

        let assembly_table_offset = file.seek(SeekFrom::Current(0))?;

        //write assembly table
        let mut pointer = Vec::new();
        for entry in assembly_table {
            if entry.pixel_src != 0 {
                pointer.push(file.seek(SeekFrom::Current(0))?);
            };
            entry.write(file)?;
        }

        Ok((assembly_table_offset, pointer))
    }*/
}
