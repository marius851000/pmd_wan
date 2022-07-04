use anyhow::{bail, Context};
use binwrite::BinWrite;
use byteorder::{ReadBytesExt, LE};
use image::{ImageBuffer, Rgba};
use std::io::{Read, Seek, SeekFrom, Write};
use thiserror::Error;

use crate::{CompressionMethod, Palette, Resolution, SpriteType, WanError};

#[derive(Error, Debug)]
pub enum ImageBytesToImageError {
    #[error("The new image you tried to create would end up with 0 pixels")]
    ZeroSizedImage,
    #[error("The image can't be created. The resolution is likely too big compared the size of this ImageBytes")]
    CantCreateImage,
    #[error("The color with the id {0} and the palette id {1} doesn't exist in the palette")]
    UnknownColor(u8, u16),
    #[error("The metaframe point to the ImageBytes {0}, which doesn't exist")]
    NoImageBytes(usize),
    #[error("Failed to decode the image")]
    CantDecodeImage(#[from] DecodeImageError),
}

#[derive(Debug)]
pub struct ImageAssemblyEntry {
    pub pixel_src: u64,
    pub pixel_amount: u32,
    pub byte_amount: u16,
    pub _z_index: u32,
}

impl ImageAssemblyEntry {
    fn new_from_bytes<F: Read>(file: &mut F) -> Result<ImageAssemblyEntry, WanError> {
        let pixel_src = file.read_u32::<LE>()? as u64;
        let byte_amount = file.read_u16::<LE>()?;
        let pixel_amount = (byte_amount as u32) * 2;
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

#[derive(PartialEq, Eq, Debug)]
pub struct ImageBytes {
    pub mixed_pixels: Vec<u8>,
    pub z_index: u32,
}

impl ImageBytes {
    pub fn new_from_bytes<F: Read + Seek>(file: &mut F) -> Result<ImageBytes, WanError> {
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
                        None => {
                            last_pointer = Some(asm_entry.pixel_src + asm_entry.byte_amount as u64)
                        }
                        Some(value) => {
                            if value == asm_entry.pixel_src {
                                last_pointer = Some(asm_entry.byte_amount as u64 + value);
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
            "the image contain {} assembly entry, with an image size of {}.",
            img_asm_table.len(),
            image_size
        );

        let mut mixed_pixels = Vec::with_capacity(64 * 64);

        let mut z_index = None;

        trace!("{:#?}", img_asm_table);

        let mut read_buffer = Vec::with_capacity(64);

        for entry in &img_asm_table {
            if entry.pixel_src == 0 {
                mixed_pixels.extend(&vec![0; entry.pixel_amount as usize]);
            } else {
                file.seek(SeekFrom::Start(entry.pixel_src))?;
                read_buffer.resize(entry.byte_amount as usize, 0);
                file.read(&mut read_buffer)?;
                for pixel_pair in &read_buffer {
                    mixed_pixels.extend(&[pixel_pair >> 4, pixel_pair & 0x0F]);
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

        if mixed_pixels.is_empty() {
            return Err(WanError::EmptyImageBytes);
        }

        //No panic : z_index is redefined whenever bytes is added to mixed_pixels, and it return earlier if that's the case
        let z_index = z_index.unwrap();

        Ok(ImageBytes {
            mixed_pixels,
            z_index,
        })
    }

    /*pub fn set_ordered_pixel() -> Result<Vec<u8>, WanError> {
        let mut pixel_list: Vec<u8> = vec![]; //a value = a pixel (acording to the tileset). None is fully transparent

        for chunk in self.pixels.chunks_exact(64) {
            for in_group_y in 0..8 {
                for in_group_x in &[1, 0, 3, 2, 5, 4, 7, 6] {
                    pixel_list.push(chunk[in_group_y * 8 + in_group_x]);
                }
            }
        }

        Ok(pixel_list)
    }*/

    pub fn write<F: Write + Seek>(
        &self,
        file: &mut F,
        sprite_type: SpriteType,
    ) -> Result<(u64, Vec<u64>), WanError> {
        let compression_method = if sprite_type == SpriteType::Chara {
            CompressionMethod::CompressionMethodOriginal
        } else {
            CompressionMethod::NoCompression
        };

        let mut assembly_table = compression_method.compress(self, &self.mixed_pixels, file)?;

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
        resolution: &Resolution,
        palette_id: u16,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBytesToImageError> {
        if resolution.x == 0 || resolution.y == 0 {
            return Err(ImageBytesToImageError::ZeroSizedImage);
        };

        let mut pixels: Vec<u8> =
            Vec::with_capacity(resolution.x as usize * resolution.y as usize * 4);

        for pixel in decode_image_pixel(&self.mixed_pixels, resolution)? {
            let mut color = if pixel == 0 {
                [0, 0, 0, 0]
            } else {
                match palette.get(pixel, palette_id) {
                    Some(c) => c,
                    None => return Err(ImageBytesToImageError::UnknownColor(pixel, palette_id)),
                }
            };
            color[3] = color[3].saturating_mul(2);
            pixels.extend(color);
        }

        let img = match ImageBuffer::from_vec(resolution.x as u32, resolution.y as u32, pixels) {
            Some(img) => img,
            None => return Err(ImageBytesToImageError::CantCreateImage),
        };

        Ok(img)
    }
}

#[derive(Error, Debug)]
pub enum DecodeImageError {
    #[error("The x resolution ({0}) isn't a multiple of 8")]
    XResolutionNotMultipleEight(u8),
    #[error("The y resolution ({0}) isn't a multiple of 8")]
    YResolutionNotMultipleEight(u8),
    #[error("The target resolution have no pixel (one of x or y resolution is 0)")]
    NoPixel,
}

/// Take the raw encoded image (from an [`ImageBytes`]), and decode them into a list of pixels
pub fn decode_image_pixel(
    pixels: &[u8],
    resolution: &Resolution,
) -> Result<Vec<u8>, DecodeImageError> {
    if resolution.x % 8 != 0 {
        return Err(DecodeImageError::XResolutionNotMultipleEight(resolution.x));
    }
    if resolution.y % 8 != 0 {
        return Err(DecodeImageError::YResolutionNotMultipleEight(resolution.y));
    }
    if resolution.x == 0 || resolution.y == 0 {
        return Err(DecodeImageError::NoPixel);
    }
    let mut dest = vec![0; resolution.x as usize * resolution.y as usize];
    let mut chunk_x = 0;
    let mut chunk_y = 0;
    let max_chunk_x = resolution.x / 8 - 1;
    'main: for chunk in pixels.chunks_exact(64) {
        let mut pixel_for_chunk = chunk.iter();
        for line in 0..8 {
            let line_start_offset = (chunk_y as usize * 8 + line as usize) * resolution.x as usize
                + chunk_x as usize * 8;
            for row_pair in 0..4 {
                //no panic : 64 elements are guaranted, and this is looped 8*4=32 times
                match dest.get_mut(line_start_offset + row_pair * 2 + 1) {
                    Some(entry) => *entry = *pixel_for_chunk.next().unwrap(),
                    None => break 'main,
                }
                dest[line_start_offset + row_pair * 2] = *pixel_for_chunk.next().unwrap();
            }
        }
        chunk_x += 1;
        if chunk_x > max_chunk_x {
            chunk_x = 0;
            chunk_y += 1;
        };
    }
    Ok(dest)
}

pub fn encode_image_pixel(pixels: &[u8], resolution: &Resolution) -> anyhow::Result<Vec<u8>> {
    if resolution.x % 8 != 0 || resolution.y % 8 != 0 {
        bail!(
            "The image resolution ({:?}) isn't a multiple of 8",
            resolution
        );
    }
    if resolution.x == 0 || resolution.y == 0 {
        bail!(
            "The image with the resolution {:?} have no pixel",
            resolution
        )
    }
    // will iterate over each line, placing them at the correct place in the output buffer
    let mut output_buffer = vec![0; resolution.x as usize * resolution.y as usize];
    let mut pixel_chunk_line_iter = pixels.chunks_exact(8);
    for chunk_column in 0..(resolution.y / 8) {
        for sub_chunk_line in 0..8 {
            for chunk_row in 0..(resolution.x / 8) {
                let chunk_row_data = pixel_chunk_line_iter
                    .next()
                    .context("The input buffer is too small")?;
                let number_chunk_ahead: usize =
                    chunk_column as usize * (resolution.x / 8) as usize + chunk_row as usize;
                let pos_total = number_chunk_ahead * 64 + sub_chunk_line * 8;
                if output_buffer.len() < pos_total + 8 {
                    bail!("The input buffer is too small")
                };
                //no panic : chunk_row_data is always of length 8
                output_buffer[pos_total] = chunk_row_data[1];
                output_buffer[pos_total + 1] = chunk_row_data[0];
                output_buffer[pos_total + 2] = chunk_row_data[3];
                output_buffer[pos_total + 3] = chunk_row_data[2];
                output_buffer[pos_total + 4] = chunk_row_data[5];
                output_buffer[pos_total + 5] = chunk_row_data[4];
                output_buffer[pos_total + 6] = chunk_row_data[7];
                output_buffer[pos_total + 7] = chunk_row_data[6];
            }
        }
    }

    Ok(output_buffer)
}
