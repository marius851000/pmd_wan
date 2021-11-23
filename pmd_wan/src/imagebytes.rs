use binwrite::BinWrite;
use byteorder::{ReadBytesExt, LE};
use image::{ImageBuffer, Rgba};
use std::io::{Read, Seek, SeekFrom, Write};
use thiserror::Error;

use crate::{CompressionMethod, Coordinate, Palette, Resolution, WanError};

#[derive(Error, Debug)]
pub enum ImageBytesToImageError {
    #[error("The new image you tried to create would end up with 0 pixels")]
    ZeroSizedImage,
    #[error("The image can't be created. The resolution is likely too big compared the size of this ImageBytes")]
    CantCreateImage,
    #[error("The color with the id {0} and the palette id {1} doesn't exist in the palette")]
    UnknownColor(u8, u16),
    #[error("The metaframe doesn't have a resolution")]
    NoResolution,
    #[error("The metaframe point to the ImageBytes {0}, which doesn't exist")]
    NoImageBytes(usize),
}

#[derive(Debug)]
pub struct ImageAssemblyEntry {
    pub pixel_src: u64,
    pub pixel_amount: u64,
    pub byte_amount: u64,
    pub _z_index: u32,
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

    pub fn write<F: Write>(&self, file: &mut F) -> Result<(), WanError> {
        (
            self.pixel_src as u32,
            self.byte_amount as u16,
            0u16,
            self._z_index,
        )
            .write(file)?;
        Ok(())
    }
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

    fn next(&mut self) -> Coordinate {
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
        Coordinate { x, y }
    }
}

#[derive(PartialEq, Eq)]
pub struct ImageBytes {
    pub pixels: Vec<u8>,
    pub z_index: u32,
}

impl ImageBytes {
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        //TODO: do not use the resolution (but instead use the end sequence symbol)
        resolution: Resolution<u8>, //resolution is required to know the size of the area (assumed to be the same for all metaframe reading it)
    ) -> Result<ImageBytes, WanError> {
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

        let mut pixels = vec![0; resolution.x as usize * resolution.y as usize];

        let mut img_pixel_pointer = ImgPixelPointer::new(resolution.x as u32, resolution.y as u32);
        let mut z_index = None;

        let test_out_of_bound = |out_resolution: &Coordinate| {
            let x_res = resolution.x as u32;
            let y_res = resolution.y as u32;
            if out_resolution.x >= x_res || out_resolution.y >= y_res {
                Err(WanError::SpriteTooSmall)
            } else {
                Ok(())
            }
        };

        trace!("{:#?}", img_asm_table);
        assert_eq!(img_asm_table.len(), 1);

        for entry in &img_asm_table {
            if entry.pixel_src == 0 {
                for _ in 0..entry.pixel_amount {
                    let pixel_pos = img_pixel_pointer.next();
                    test_out_of_bound(&pixel_pos)?;
                    pixels[pixel_pos.y as usize * resolution.x as usize + pixel_pos.x as usize] = 0;
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
                    pixels[pixel_pos.y as usize * resolution.x as usize + pixel_pos.x as usize] =
                        color_id;
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

        Ok(ImageBytes { pixels, z_index })
    }

    fn get_pixel_list(&self) -> Result<Vec<u8>, WanError> {
        let mut pixel_list: Vec<u8> = vec![]; //a value = a pixel (acording to the tileset). None is fully transparent

        for chunk in self.pixels.chunks_exact(64) {
            for in_group_y in 0..8 {
                for in_group_x in &[1, 0, 3, 2, 5, 4, 7, 6] {
                    pixel_list.push(chunk[in_group_y * 8 + in_group_x]);
                }
            }
        }

        Ok(pixel_list)
    }

    //TODO: check this is actually valid
    pub fn write<F: Write + Seek>(&self, file: &mut F) -> Result<(u64, Vec<u64>), WanError> {
        let pixel_list = self.get_pixel_list()?;

        let compression_method = CompressionMethod::NoCompression;

        let mut assembly_table = compression_method.compress(self, &pixel_list, file)?;

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
    }

    pub fn get_image(
        &self,
        palette: &Palette,
        resolution: &Resolution<u8>,
        palette_id: u16,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBytesToImageError> {
        if resolution.x == 0 || resolution.y == 0 {
            return Err(ImageBytesToImageError::ZeroSizedImage);
        };

        let mut pixels: Vec<u8> =
            Vec::with_capacity(resolution.x as usize * resolution.y as usize * 4);
        for pixel in &self.pixels {
            let color = match palette.get(*pixel, palette_id) {
                Some(c) => c,
                None => return Err(ImageBytesToImageError::UnknownColor(*pixel, palette_id)),
            };
            let new_alpha = color.3.saturating_mul(2);
            pixels.extend([color.0, color.1, color.2, new_alpha]);
        }

        let img = match ImageBuffer::from_vec(resolution.x as u32, resolution.y as u32, pixels) {
            Some(img) => img,
            None => return Err(ImageBytesToImageError::CantCreateImage),
        };

        Ok(img)
    }
}
