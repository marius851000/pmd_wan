#![allow(dead_code)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::cognitive_complexity)]

extern crate image;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub enum WanError {
    IOError(io::Error),
    ImageIDPointBackButFirstImage,
    MetaFrameLessThanLessOne(i16),
    InvalidOffset,
    InvalidResolution,
    IncoherentPointerToImagePart,
    NoZIndex,
    ImpossibleAlphaLevel,
    NullImagePointer,
    ImageWithoutResolution,
    PaletteDontEndWithZero,
    PaletteOOB,
    CantFindColorInPalette,
    InvalidSir0([u8; 4]),
    InvalidEndOfSir0Header([u8; 4]),
    TypeOfSpriteUnknown(u16),
    InvalidColorNumber(u16),
}

impl From<io::Error> for WanError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl Error for WanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IOError(err) => Some(err),
            _ => None,
        }
    }
}
impl fmt::Display for WanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(_) => write!(f, "an io error happened"),
            Self::ImageIDPointBackButFirstImage => write!(f, "an image id point to the same than the previous one, but it is the first image"),
            Self::MetaFrameLessThanLessOne(id) => write!(f, "a metaframe is inferior to -1, but that is not valid (it actually is {})", id),
            Self::InvalidOffset => write!(f, "in the creation of a meta frame store: the check for the offset of the pointer of the animation group are not valid!"),
            Self::InvalidResolution => write!(f, "the resolution was not found!!!"),
            Self::IncoherentPointerToImagePart => write!(f, "pointer to image parts are not coherent."),
            Self::NoZIndex => write!(f, "impossible to find a definied z_index (the image is probably empty)!!! aborting"),
            Self::ImpossibleAlphaLevel => write!(f, "an impossible alpha level was found in the picture !"),
            Self::NullImagePointer => write!(f, "an image data pointer is null !!!"),
            Self::ImageWithoutResolution => write!(f, "the image does not have a resolution"),
            Self::PaletteDontEndWithZero => write!(f, "the palette data doesn't end with 0s !!!"),
            Self::PaletteOOB => write!(f, "impossible to get a color in the palette, as this will do an OOB."),
            Self::CantFindColorInPalette => write!(f, "impossible to find the specified color in the palette!"),
            Self::InvalidSir0(value) => write!(f, "the sir0 header in invalid, expected SIR0, found {:?}", value),
            Self::InvalidEndOfSir0Header(value) => write!(f, "the end of the sir0 header should be four 0, found {:?}", value),
            Self::TypeOfSpriteUnknown(value) => write!(f, "the type of sprite is unknown (found the sprite type id {}, but this program only known sprite for [0, 1, 3])", value),
            Self::InvalidColorNumber(value) => write!(f, "the 2 byte that indicate the number of color is invalid (found {}, expected 0 or 1)", value),
        }
    }
}

fn get_bit_u16(byte: u16, id: u16) -> Option<bool> {
    if id < 8 {
        Some((byte >> (15 - id) << 15) >= 1)
    } else {
        None
    }
}

fn wan_read_i16<F: Read>(file: &mut F) -> Result<i16, WanError> {
    let mut buffer = [0; 2];
    file.read_exact(&mut buffer)?;
    Ok(i16::from_le_bytes(buffer))
}

fn wan_read_u8<F: Read>(file: &mut F) -> Result<u8, WanError> {
    let mut buffer = [0];
    file.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

fn wan_read_u16<F: Read>(file: &mut F) -> Result<u16, WanError> {
    let mut buffer = [0; 2];
    file.read_exact(&mut buffer)?;
    Ok(u16::from_le_bytes(buffer))
}

fn wan_read_u32<F: Read>(file: &mut F) -> Result<u32, WanError> {
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn wan_read_raw_4<F: Read>(file: &mut F) -> Result<[u8; 4], WanError> {
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn wan_write_i16<F: Write>(file: &mut F, value: i16) -> Result<(), WanError> {
    Ok(file.write_all(&value.to_le_bytes())?)
}

fn wan_write_u16<F: Write>(file: &mut F, value: u16) -> Result<(), WanError> {
    Ok(file.write_all(&value.to_le_bytes())?)
}

fn wan_write_u32<F: Write>(file: &mut F, value: u32) -> Result<(), WanError> {
    Ok(file.write_all(&value.to_le_bytes())?)
}

fn wan_write_u8<F: Write>(file: &mut F, value: u8) -> Result<(), WanError> {
    Ok(file.write_all(&value.to_le_bytes())?)
}

#[derive(Debug, PartialEq)]
pub enum SpriteType {
    PropsUI,
    Chara,
    Unknown,
}

impl SpriteType {
    fn get_id(&self) -> u8 {
        match self {
            SpriteType::PropsUI => 0,
            SpriteType::Chara => 1,
            SpriteType::Unknown => 3,
        }
    }
}

#[derive(Debug)]
pub struct MetaFrame {
    pub unk1: u16,
    pub unk2: u16,
    pub unk3: bool,
    pub image_index: usize,
    pub offset_y: i32,
    pub offset_x: i32,
    pub is_last: bool,
    pub v_flip: bool,
    pub h_flip: bool,
    pub is_mosaic: bool,
    pub pal_idx: u16,
    pub resolution: Option<Resolution<u8>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Resolution<T> {
    pub x: T,
    pub y: T,
}

impl Resolution<u8> {
    pub fn get_indice(self) -> (u16, u16) {
        match (self.y, self.x) {
            (8, 8) => (0, 0),
            (16, 16) => (0, 1),
            (32, 32) => (0, 2),
            (64, 64) => (0, 3),
            (16, 8) => (1, 0),
            (8, 16) => (2, 0),
            (32, 8) => (1, 1),
            (8, 32) => (2, 1),
            (32, 16) => (1, 2),
            (16, 32) => (2, 2),
            (64, 32) => (1, 3),
            (32, 64) => (2, 3),
            _ => panic!(),
        }
    }
}

impl MetaFrame {
    fn new_from_bytes<F: Read>(
        file: &mut F,
        previous_image: Option<usize>,
    ) -> Result<MetaFrame, WanError> {
        trace!("parsing a meta-frame");
        let image_index = match wan_read_i16(file)? {
            -1 => match previous_image {
                None => return Err(WanError::ImageIDPointBackButFirstImage),
                Some(value) => value,
            },
            x => {
                if x >= 0 {
                    x as usize
                } else {
                    return Err(WanError::MetaFrameLessThanLessOne(x));
                }
            }
        };

        let unk1 = wan_read_u16(file)?;

        // they are quite strangely encoded (the fact I should read as little-endian the 2 byte correctly reading them)

        // bit in ppmdu tool are right to left !!!
        let offset_y_data = wan_read_u16(file)?;
        let size_indice_y = ((0xC000 & offset_y_data) >> (8 + 6)) as u8;
        let is_mosaic = get_bit_u16(offset_y_data, 3).unwrap();
        let unk3 = get_bit_u16(offset_y_data, 7).unwrap();
        let offset_y = i16::from_le_bytes((offset_y_data & 0x00FF).to_le_bytes()) as i32; //range: 0-255

        let offset_x_data = wan_read_u16(file)?;
        let size_indice_x = ((0xC000 & offset_x_data) >> (8 + 6)) as u8;
        let v_flip = get_bit_u16(offset_x_data, 2).unwrap();
        let h_flip = get_bit_u16(offset_x_data, 3).unwrap();
        let is_last = get_bit_u16(offset_x_data, 4).unwrap();
        let offset_x = (i16::from_le_bytes((offset_x_data & 0x01FF).to_le_bytes()) as i32) - 256; //range: 0-511

        let unk2 = wan_read_u16(file)?;
        let pal_idx = ((0xF000 & unk2) >> 12) as u16;

        Ok(MetaFrame {
            unk1,
            unk2,
            unk3,
            image_index,
            offset_x,
            offset_y: if offset_y > 128 {
                offset_y - 256
            } else {
                offset_y
            },
            is_last,
            v_flip,
            h_flip,
            is_mosaic,
            pal_idx: pal_idx,
            resolution: match (size_indice_y << 4) + size_indice_x {
                0x00 => Some(Resolution { x: 8, y: 8 }),
                0x01 => Some(Resolution { x: 16, y: 16 }),
                0x02 => Some(Resolution { x: 32, y: 32 }),
                0x03 => Some(Resolution { x: 64, y: 64 }),
                0x10 => Some(Resolution { x: 16, y: 8 }),
                0x20 => Some(Resolution { x: 8, y: 16 }),
                0x11 => Some(Resolution { x: 32, y: 8 }),
                0x21 => Some(Resolution { x: 8, y: 32 }),
                0x12 => Some(Resolution { x: 32, y: 16 }),
                0x22 => Some(Resolution { x: 16, y: 32 }),
                0x13 => Some(Resolution { x: 64, y: 32 }),
                0x23 => Some(Resolution { x: 32, y: 64 }),
                _ => None, // seem to be normal
            },
        })
    }

    fn is_last(&self) -> bool {
        self.is_last
    }

    fn write<F: Write>(
        file: &mut F,
        meta_frame: &MetaFrame,
        previous_image: Option<usize>,
    ) -> Result<(), WanError> {
        let image_index: i16 = match previous_image {
            None => meta_frame.image_index as i16,
            Some(value) => {
                if meta_frame.image_index == value {
                    -1
                } else {
                    meta_frame.image_index as i16
                }
            }
        };

        wan_write_i16(file, image_index)?;

        wan_write_u16(file, meta_frame.unk1)?; //unk

        let (size_indice_y, size_indice_x) = match meta_frame.resolution {
            Some(value) => value.get_indice(),
            None => panic!(),
        };

        let offset_y_data: u16 = (size_indice_y << 13)
            + if meta_frame.is_mosaic { 1 << 12 } else { 0 }
            + ((meta_frame.unk3 as u16) << (16 - 7 - 1))
            + (u16::from_le_bytes((meta_frame.offset_y as i16).to_le_bytes()) & 0x00FF);
        wan_write_u16(file, offset_y_data)?;

        let offset_x_data: u16 = (size_indice_x << 14)
            + ((meta_frame.v_flip as u16) << (16 - 2 - 1))
            + ((meta_frame.h_flip as u16) << (16 - 3 - 1))
            + ((meta_frame.is_last as u16) << (16 - 4 - 1))
            + (u16::from_le_bytes(((meta_frame.offset_x + 256) as i16).to_le_bytes()) & 0x01FF);

        wan_write_u16(file, offset_x_data)?;

        wan_write_u16(file, meta_frame.unk2)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct MetaFrameGroup {
    pub meta_frames_id: Vec<usize>,
}

impl MetaFrameGroup {
    fn new_from_bytes<F: Read>(
        file: &mut F,
        meta_frames: &mut Vec<MetaFrame>,
    ) -> Result<MetaFrameGroup, WanError> {
        let mut meta_frames_id = Vec::new();
        let mut previous_image = None;
        loop {
            meta_frames_id.push(meta_frames.len()); // We refer to the metaframe we will put here
            let meta_frame = MetaFrame::new_from_bytes(file, previous_image)?;
            previous_image = Some(meta_frame.image_index);
            meta_frames.push(meta_frame);
            trace!("it's data: {:?}", meta_frames[meta_frames.len() - 1]);
            if meta_frames[meta_frames.len() - 1].is_last() {
                break;
            }
        }
        Ok(MetaFrameGroup {
            meta_frames_id,
        })
    }

    fn write<F: Write>(
        file: &mut F,
        meta_frame_group: &MetaFrameGroup,
        meta_frames: &[MetaFrame],
    ) -> Result<(), WanError> {
        let mut previous_image: Option<usize> = None;
        for l in 0..meta_frame_group.meta_frames_id.len() {
            let meta_frames_id = meta_frame_group.meta_frames_id[l];
            let meta_frame_to_write = &meta_frames[meta_frames_id];
            MetaFrame::write(file, meta_frame_to_write, previous_image)?;
            previous_image = Some(l);
        }
        Ok(())
    }
}
pub struct MetaFrameStore {
    pub meta_frames: Vec<MetaFrame>,
    pub meta_frame_groups: Vec<MetaFrameGroup>,
}

impl MetaFrameStore {
    // assume that the pointer is already well positionned
    fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        nb_meta_frame: u64,
    ) -> Result<MetaFrameStore, WanError> {
        let mut meta_frames = Vec::new();
        let mut meta_frame_groups = Vec::new();
        let mut last_pointer = None;

        let mut meta_frame_reference: Vec<u64> = Vec::new();
        for _ in 0..nb_meta_frame {
            let actual_ptr = wan_read_u32(file)? as u64;
            //some check
            match last_pointer {
                None => last_pointer = Some(actual_ptr),
                Some(value) => {
                    if (actual_ptr - value) % 10 != 0 {
                        return Err(WanError::InvalidOffset);
                    }
                }
            };
            meta_frame_reference.push(actual_ptr);
        }

        for meta_frame_id in 0..nb_meta_frame {
            trace!(
                "parsing meta-frame n°{} (at offset {})",
                meta_frame_id,
                meta_frame_reference[meta_frame_id as usize]
            );
            file.seek(SeekFrom::Start(
                meta_frame_reference[meta_frame_id as usize],
            ))?;
            meta_frame_groups.push(MetaFrameGroup::new_from_bytes(file, &mut meta_frames)?);
        }
        Ok(MetaFrameStore {
            meta_frames,
            meta_frame_groups,
        })
    }

    fn find_resolution_and_pal_idx_image(&self, image_id: u32) -> Result<(Option<Resolution<u8>>, u16), WanError> {
        for actual_image in &self.meta_frames {
            if actual_image.image_index == image_id as usize {
                return Ok((actual_image.resolution, actual_image.pal_idx));
            };
        }
        Err(WanError::InvalidResolution)
    }

    fn write<F: Write + Seek>(
        file: &mut F,
        meta_frame_store: &MetaFrameStore,
    ) -> Result<Vec<u32>, WanError> {
        let nb_meta_frame = meta_frame_store.meta_frame_groups.len();
        let mut meta_frame_references = vec![];

        for l in 0..nb_meta_frame {
            meta_frame_references.push(file.seek(SeekFrom::Current(0))? as u32);
            MetaFrameGroup::write(
                file,
                &meta_frame_store.meta_frame_groups[l],
                &meta_frame_store.meta_frames,
            )?;
        }

        Ok(meta_frame_references)
    }
}

#[derive(Debug)]
pub struct ImageAssemblyEntry {
    pixel_src: u64,
    pixel_amount: u64,
    byte_amount: u64,
    _z_index: u32,
}

impl ImageAssemblyEntry {
    fn new_from_bytes<F: Read>(file: &mut F) -> Result<ImageAssemblyEntry, WanError> {
        let pixel_src = wan_read_u32(file)? as u64;
        let byte_amount = wan_read_u16(file)? as u64;
        let pixel_amount = byte_amount * 2;
        wan_read_u16(file)?;
        let z_index = wan_read_u32(file)?;
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

    fn write<F: Write>(&self, file: &mut F) -> Result<(), WanError> {
        wan_write_u32(file, self.pixel_src as u32)?;
        wan_write_u16(file, self.byte_amount as u16)?;
        wan_write_u16(file, 0)?;
        wan_write_u32(file, self._z_index)?;
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
    fn new_from_bytes<F: Read + Seek>(
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
        for entry in &img_asm_table {
            if entry.pixel_src == 0 {
                for _ in 0..entry.pixel_amount {
                    let pixel_pos = img_pixel_pointer.next();
                    let pixel = img.get_pixel_mut(pixel_pos.x, pixel_pos.y);
                    *pixel = image::Rgba([0, 0, 0, 0]);
                }
            } else {
                file.seek(SeekFrom::Start(entry.pixel_src))?;
                let mut actual_byte = 0;
                for loop_id in 0..entry.pixel_amount {
                    let color_id = if loop_id % 2 == 0 {
                        actual_byte = wan_read_u8(file)?;
                        actual_byte >> 4
                    } else {
                        (actual_byte << 4) >> 4
                    };
                    let pixel_pos = img_pixel_pointer.next();
                    let pixel = img.get_pixel_mut(pixel_pos.x, pixel_pos.y);
                    if color_id == 0 {
                        *pixel = image::Rgba([0, 0, 0, 0]);
                    } else {
                        let color = palette.get((pal_idx * 16 + color_id as u16) as usize)?;
                        let color = [color.0, color.1, color.2, 255];
                        *pixel = image::Rgba(color);
                    };
                }
            }
            match z_index {
                None => z_index = Some(entry._z_index),
                Some(index) => {
                    debug_assert!(index == entry._z_index);
                    z_index = Some(entry._z_index);
                }
            }
        }

        let z_index = match z_index {
            Some(value) => value,
            None => return Err(WanError::NoZIndex),
        };

        Ok(Image { img, z_index })
    }

    fn write<F: Write + Seek>(
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
    }
}

pub struct ImageStore {
    pub images: Vec<Image>,
}

impl ImageStore {
    fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        amount_images: u32,
        meta_frame_store: &MetaFrameStore,
        palette: &Palette,
    ) -> Result<ImageStore, WanError> {
        trace!("will read {} image", amount_images);
        let mut image_pointers: Vec<u64> = Vec::new(); //list of reference to image
        for _ in 0..amount_images {
            let current_pointer = wan_read_u32(file)? as u64;
            if current_pointer == 0 {
                return Err(WanError::NullImagePointer);
            };
            image_pointers.push(current_pointer);
        }

        trace!("reading the image table");
        let mut images = Vec::new();

        for (image_id, image) in image_pointers.iter().enumerate() {
            trace!("reading image n°{}", image_id);
            let (resolution, pal_idx) = meta_frame_store.find_resolution_and_pal_idx_image(image_id as u32)?;
            if resolution.is_none() {
                return Err(WanError::ImageWithoutResolution)
            }
            file.seek(SeekFrom::Start(*image))?;
            let img = Image::new_from_bytes(file, resolution.unwrap(), pal_idx, &palette)?;
            images.push(img);
        }

        Ok(ImageStore { images })
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    fn write<F: Write + Seek>(
        file: &mut F,
        wanimage: &WanImage,
    ) -> Result<(Vec<u64>, Vec<u64>), WanError> {
        let mut image_offset = vec![];
        let mut sir0_pointer_images = vec![];

        for image in &wanimage.image_store.images {
            let (assembly_table_offset, sir0_img_pointer) = image.write(file, &wanimage.palette)?;
            for pointer in sir0_img_pointer {
                sir0_pointer_images.push(pointer)
            }
            image_offset.push(assembly_table_offset);
        }
        Ok((image_offset, sir0_pointer_images))
    }
}

pub struct Palette {
    pub palette: Vec<(u8, u8, u8, u8)>,
}

impl Palette {
    fn new_from_bytes<F: Read + Seek>(file: &mut F) -> Result<Palette, WanError> {
        let mut palette = Vec::new();
        let pointer_palette_start = wan_read_u32(file)? as u64;
        wan_read_u16(file)?;
        let nb_color = wan_read_u16(file)?;
        wan_read_raw_4(file)?;
        trace!(
            "palette_start: {}, nb_color: {}",
            pointer_palette_start,
            nb_color
        );
        if wan_read_u32(file)? != 0 {
            return Err(WanError::PaletteDontEndWithZero);
        };
        file.seek(SeekFrom::Start(pointer_palette_start))?;
        for _ in 0..nb_color {
            let red = wan_read_u8(file)?;
            let green = wan_read_u8(file)?;
            let blue = wan_read_u8(file)?;
            let alpha = wan_read_u8(file)?;
            palette.push((red, green, blue, alpha));
        }
        Ok(Palette { palette })
    }

    fn get(&self, id: usize) -> Result<(u8, u8, u8, u8), WanError> {
        if id >= self.palette.len() {
            return Err(WanError::PaletteOOB);
        };
        Ok(self.palette[id])
    }

    fn color_id(&self, target_color: (u8, u8, u8, u8)) -> Result<usize, WanError> {
        for color_id in 0..self.palette.len() {
            if self.palette[color_id] == target_color {
                return Ok(color_id);
            }
        }
        error!("impossible to find the palette {:?}", target_color);
        error!("tried :");
        for color in &self.palette {
            error!("{:?}", color);
        }
        Err(WanError::CantFindColorInPalette)
    }

    fn write<F: Write + Seek>(&self, file: &mut F) -> Result<u64, WanError> {
        let palette_start_offset = file.seek(SeekFrom::Current(0))?;
        for color in &self.palette {
            wan_write_u8(file, color.0)?;
            wan_write_u8(file, color.1)?;
            wan_write_u8(file, color.2)?;
            wan_write_u8(file, color.3)?;
        }

        let palette_header_offset = file.seek(SeekFrom::Current(0))?;
        wan_write_u32(file, palette_start_offset as u32)?;
        wan_write_u16(file, 0)?; //unk
        wan_write_u16(file, 16)?; //TODO: assume the picture is 4bit
        wan_write_u32(file, 0xFF << 16)?; //unk
        wan_write_u32(file, 0)?; //magic

        Ok(palette_header_offset)
    }
}

#[derive(Debug)]
struct AnimGroupEntry {
    pointer: u32,
    group_lenght: u16,
    _unk16: u16,
    id: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnimationFrame {
    pub duration: u8,
    pub flag: u8,
    pub frame_id: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub shadow_offset_x: i16,
    pub shadow_offset_y: i16,
}

impl AnimationFrame {
    fn new<F: Read>(file: &mut F) -> Result<AnimationFrame, WanError> {
        let duration = wan_read_u8(file)?;
        let flag = wan_read_u8(file)?;
        let frame_id = wan_read_u16(file)?;
        let offset_x = wan_read_i16(file)?;
        let offset_y = wan_read_i16(file)?;
        let shadow_offset_x = wan_read_i16(file)?;
        let shadow_offset_y = wan_read_i16(file)?;
        Ok(AnimationFrame {
            duration,
            flag,
            frame_id,
            offset_x,
            offset_y,
            shadow_offset_x,
            shadow_offset_y,
        })
    }

    fn is_null(&self) -> bool {
        self.duration == 0 && self.frame_id == 0
    }

    pub fn write<F: Write>(file: &mut F, frame: &AnimationFrame) -> Result<(), WanError> {
        wan_write_u8(file, frame.duration)?;
        wan_write_u8(file, frame.flag)?;
        wan_write_u16(file, frame.frame_id)?;
        wan_write_i16(file, frame.offset_x)?;
        wan_write_i16(file, frame.offset_y)?;
        wan_write_i16(file, frame.shadow_offset_x)?;
        wan_write_i16(file, frame.shadow_offset_y)?;
        Ok(())
    }

    pub fn write_null<F: Write>(file: &mut F) -> Result<(), WanError> {
        AnimationFrame::write(
            file,
            &AnimationFrame {
                duration: 0,
                flag: 0,
                frame_id: 0,
                offset_x: 0,
                offset_y: 0,
                shadow_offset_x: 0,
                shadow_offset_y: 0,
            },
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
}

impl Animation {
    fn new<F: Read>(file: &mut F) -> Result<Animation, WanError> {
        let mut frames = Vec::new();
        loop {
            let current_frame = AnimationFrame::new(file)?;
            if current_frame.is_null() {
                break;
            }
            frames.push(current_frame);
        }
        Ok(Animation { frames })
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn write<F: Write>(file: &mut F, animation: &Animation) -> Result<(), WanError> {
        for frame in &animation.frames {
            AnimationFrame::write(file, frame)?;
        }
        AnimationFrame::write_null(file)?;
        Ok(())
    }
}

pub struct AnimStore {
    pub animations: Vec<Animation>,
    pub copied_on_previous: Option<Vec<bool>>, //indicate if a sprite can copy on the previous. Will always copy if possible if None
    pub anim_groups: Vec<Option<(usize, usize)>>, //usize1 = start, usize2 = lenght
}

impl AnimStore {
    fn new<F: Read + Seek>(
        file: &mut F,
        pointer_animation_groups_table: u64,
        amount_animation_group: u16,
        is_for_chara: bool
    ) -> Result<(AnimStore, u64), WanError> {
        //TODO: rewrite this function, it seem to be too complicated to understand
        file.seek(SeekFrom::Start(pointer_animation_groups_table))?;
        let mut anim_group_entry: Vec<Option<AnimGroupEntry>> = Vec::new();
        let add_for_chara = if is_for_chara { 7 } else { 0 };
        for animation_group_id in 0..amount_animation_group + add_for_chara {
            //HACK: CRITICAL:
            let pointer = wan_read_u32(file)?;
            if pointer == 0 {
                anim_group_entry.push(None);
                continue;
            };
            let group_lenght = wan_read_u16(file)?;
            let _unk16 = wan_read_u16(file)?;
            anim_group_entry.push(Some(AnimGroupEntry {
                pointer,
                group_lenght,
                _unk16,
                id: animation_group_id,
            }));
        }

        let mut anim_groups: Vec<Option<Vec<u64>>> = Vec::new();
        let mut particule_table_end = None;
        for anim_group_option in anim_group_entry {
            match anim_group_option {
                None => anim_groups.push(None),
                Some(anim_group) => {
                    file.seek(SeekFrom::Start(anim_group.pointer as u64))?;
                    match particule_table_end {
                        Some(value) => {
                            if file.seek(SeekFrom::Current(0))? < value {
                                particule_table_end = Some(file.seek(SeekFrom::Current(0))?);
                            }
                        }
                        None => particule_table_end = Some(file.seek(SeekFrom::Current(0))?),
                    };

                    let mut anim_ref = Vec::new();
                    for _ in 0..anim_group.group_lenght {
                        anim_ref.push(wan_read_u32(file)? as u64);
                    }
                    trace!(
                        "reading an animation group entry, id is {}, the pointer is {:?}",
                        anim_group.id,
                        anim_ref
                    );
                    anim_groups.push(Some(anim_ref));
                }
            };
        }
        if particule_table_end.is_none() {
            particule_table_end = Some(file.seek(SeekFrom::Current(0))?);
        };

        let mut animations: Vec<Animation> = Vec::new();
        let mut copied_on_previous = Vec::new();
        let mut anim_groups_result = Vec::new();
        let mut check_last_anim_pos = 0;
        for anim_group in anim_groups {
            match anim_group {
                None => anim_groups_result.push(None),
                Some(anim_group_table) => {
                    anim_groups_result.push(Some((animations.len(), anim_group_table.len())));
                    for animation in anim_group_table {
                        file.seek(SeekFrom::Start(animation))?;
                        //TODO: what is this error ?
                        /*if check_last_anim_pos > file.seek(SeekFrom::Current(0))? {
                            bail!("The check for the order of animation haven't verified.")
                        };*/
                        copied_on_previous
                            .push(file.seek(SeekFrom::Current(0))? == check_last_anim_pos);
                        check_last_anim_pos = file.seek(SeekFrom::Current(0))?;
                        animations.push(Animation::new(file)?);
                    }
                }
            };
        }

        Ok((
            AnimStore {
                animations,
                copied_on_previous: Some(copied_on_previous),
                anim_groups: anim_groups_result,
            },
            particule_table_end.unwrap(),
        ))
    }

    pub fn len(&self) -> usize {
        self.animations.len()
    }

    fn write<F: Write + Seek>(file: &mut F, anim_store: &AnimStore) -> Result<Vec<u64>, WanError> {
        let mut animations_pointer = vec![];
        let mut previous_animation: Option<&Animation> = None;
        let mut previous_pointer = None;

        for loop_nb in 0..anim_store.animations.len() {
            let animation = &anim_store.animations[loop_nb];
            let can_copy_on_previous = match &anim_store.copied_on_previous {
                None => true,
                Some(value) => value[loop_nb],
            };
            let actual_pointer = file.seek(SeekFrom::Current(0))?;

            if can_copy_on_previous {
                if let Some(p_anim) = previous_animation {
                    if *p_anim == *animation {
                        animations_pointer.push(previous_pointer.unwrap());
                        continue;
                    }
                };
            };

            animations_pointer.push(actual_pointer);
            Animation::write(file, animation)?;
            previous_animation = Some(animation);
            previous_pointer = Some(actual_pointer);
        }
        Ok(animations_pointer)
    }

    fn write_animation_group<F: Write + Seek>(
        &self,
        file: &mut F,
        animations_pointer: &[u64],
    ) -> Result<(u64, Vec<u64>), WanError> {
        let mut sir0_animation = Vec::new();

        struct AnimGroupData {
            pointer: u32,
            lenght: u16,
        }

        let mut anim_group_data: Vec<Option<AnimGroupData>> = Vec::new();
        for anim_group in &self.anim_groups {
            match anim_group {
                None => {
                    anim_group_data.push(None);
                    wan_write_u16(file, 0)?;
                }
                Some(value) => {
                    anim_group_data.push(Some(AnimGroupData {
                        pointer: file.seek(SeekFrom::Current(0))? as u32,
                        lenght: value.1 as u16,
                    }));
                    for anim_pos in 0..value.1 {
                        sir0_animation.push(file.seek(SeekFrom::Current(0))?);
                        let value_to_write =
                            animations_pointer[(value.0 as usize) + anim_pos] as u32;
                        wan_write_u32(file, value_to_write)?;
                    }
                }
            }
        }

        let animation_group_reference_offset = file.seek(SeekFrom::Current(0))?;

        for actual_data in anim_group_data {
            match actual_data {
                None => wan_write_u32(file, 0)?,
                Some(data) => {
                    sir0_animation.push(file.seek(SeekFrom::Current(0))?);
                    wan_write_u32(file, data.pointer)?;
                    wan_write_u16(file, data.lenght)?;
                    wan_write_u16(file, 0)?;
                }
            };
        }

        Ok((animation_group_reference_offset, sir0_animation))
    }
}

pub struct WanImage {
    pub image_store: ImageStore,
    pub meta_frame_store: MetaFrameStore,
    pub anim_store: AnimStore,
    pub palette: Palette,
    pub raw_particule_table: Vec<u8>,
    /// true if the picture have 256 color, false if it only have 16
    pub is_256_color: bool,
    pub sprite_type: SpriteType,
    pub unk_1: u32,
}

impl WanImage {
    pub fn new<F: Read + Seek>(b: F) -> Result<WanImage, WanError> {
        WanImage::decode_wan(b)
    }

    /// parse an image in the wan/wat format stored in the input file
    /// It assume that the file is decompressed
    pub fn decode_wan<F: Read + Seek>(mut file: F) -> Result<WanImage, WanError> {
        file.seek(SeekFrom::Start(0))?;
        debug!("start to decode a wan image");

        // first step: decode the sir0 header
        trace!("decoding the sir0 header");
        let sir0_header = wan_read_raw_4(&mut file)?;
        if sir0_header != [0x53, 0x49, 0x52, 0x30] {
            return Err(WanError::InvalidSir0(sir0_header));
        };
        let sir0_pointer_header = wan_read_u32(&mut file)? as u64;
        let _sir0_pointer_offset = wan_read_u32(&mut file)? as u64;

        let sir0_header_end = wan_read_raw_4(&mut file)?;
        if sir0_header_end != [0, 0, 0, 0] {
            return Err(WanError::InvalidEndOfSir0Header(sir0_header_end));
        };

        // second step: decode the wan header
        trace!("reading the wan header");
        file.seek(SeekFrom::Start(sir0_pointer_header))?;
        let pointer_to_anim_info = wan_read_u32(&mut file)? as u64;
        let pointer_to_image_data_info = wan_read_u32(&mut file)? as u64;
        let sprite_type = match wan_read_u16(&mut file)? {
            0 => SpriteType::PropsUI,
            1 => SpriteType::Chara,
            3 => SpriteType::Unknown,
            value => return Err(WanError::TypeOfSpriteUnknown(value)),
        };
        //unk #12
        //TODO
        /*if file.seek(SeekFrom::Current(0))? != sir0_pointer_header+10 { // an assertion
            bail!("we are not at the good position after the wan header!!!");
        }*/

        // third step: decode animation info block
        trace!("reading the animation info block");
        file.seek(SeekFrom::Start(pointer_to_anim_info))?;
        let pointer_meta_frame_reference_table = wan_read_u32(&mut file)? as u64;
        let pointer_particule_offset_table = wan_read_u32(&mut file)? as u64;
        let pointer_animation_groups_table = wan_read_u32(&mut file)? as u64;
        let amount_animation_group = wan_read_u16(&mut file)?;

        //TODO
        /*if file.seek(SeekFrom::Current(0))? != pointer_to_anim_info + 14 {
            bail!("we are not at the good position after the animation info block!!!");
        };*/
        let unk_1 = wan_read_u32(&mut file)?;

        // fourth: decode image data info
        trace!("reading the image data info");
        file.seek(SeekFrom::Start(pointer_to_image_data_info))?;
        let pointer_image_data_pointer_table = wan_read_u32(&mut file)? as u64;
        let pointer_palette = wan_read_u32(&mut file)? as u64;
        wan_read_u16(&mut file)?; //unk
        let is_256_color = match wan_read_u16(&mut file)? {
            0 => false,
            1 => true,
            color_id => return Err(WanError::InvalidColorNumber(color_id)),
        };
        wan_read_u16(&mut file)?; //unk
        let amount_images = wan_read_u16(&mut file)?;

        trace!("parsing the palette");
        //TODO
        /*if pointer_palette == 0 {
            bail!("the palette pointer is equal to 0 !!!");
        };*/
        file.seek(SeekFrom::Start(pointer_palette))?;
        let palette = Palette::new_from_bytes(&mut file)?;

        // decode meta-frame
        trace!("decoding meta-frame");
        let meta_frame_reference_end_pointer: u64 = match pointer_particule_offset_table {
            0 => match WanImage::find_first_non_null_animation_seq_entry(&mut file, pointer_animation_groups_table) {
                Some(v) => v,
                // Fall back to animation group offset
                None => pointer_animation_groups_table
            },
            value => value,
        };
        let amount_meta_frame =
            (meta_frame_reference_end_pointer - pointer_meta_frame_reference_table) / 4;
        file.seek(SeekFrom::Start(pointer_meta_frame_reference_table))?;
        let meta_frame_store = MetaFrameStore::new_from_bytes(&mut file, amount_meta_frame)?;

        // decode image
        trace!("reading the image data pointer table");
        file.seek(SeekFrom::Start(pointer_image_data_pointer_table))?;
        let image_store = ImageStore::new_from_bytes(
            &mut file,
            amount_images as u32,
            &meta_frame_store,
            &palette,
        )?;

        // decode animation
        let (anim_store, particule_table_end) = AnimStore::new(
            &mut file,
            pointer_animation_groups_table,
            amount_animation_group,
            sprite_type == SpriteType::Chara
        )?;

        let mut raw_particule_table: Vec<u8>;
        if pointer_particule_offset_table > 0 {
            file.seek(SeekFrom::Start(pointer_particule_offset_table))?;
            raw_particule_table = vec![0; (particule_table_end - pointer_particule_offset_table) as usize];
            file.read_exact(&mut raw_particule_table)?;
        } else {
            raw_particule_table = Vec::new();
        }

        Ok(WanImage {
            image_store,
            meta_frame_store,
            anim_store,
            palette,
            raw_particule_table,
            is_256_color,
            sprite_type,
            unk_1,
        })
    }

    /// If the file doesn't have an entity effect particle list, we ned to instead search
    /// for the pointer to the first animation sequence, to get the end of the meta frame table.
    fn find_first_non_null_animation_seq_entry<F: Read + Seek>(
        file: &mut F,
        pointer_animation_groups_table: u64
    ) -> Option<u64> {
        file.seek(SeekFrom::Start(pointer_animation_groups_table)).ok()?;
        loop {
            match wan_read_u32(file) {
                Ok(pntr) => {
                    if pntr != 0 {
                        return Some(pntr as u64);
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
        None
    }

    //TODO:
    /* pub fn create_wan<F: Write + Seek>(wanimage: &WanImage, file: &mut F) -> Result<(), WanError> {
        //TODO: transform all unwrap to chain_error error
        debug!("start creating a wan image");

        let mut sir0_offsets = vec![];
        // create the sir0 header
        trace!("creating the sir0 header");
        file.write(vec!(0x53, 0x49, 0x52, 0x30)).unwrap(); // sir0 magic

        let sir0_pointer_header = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        wan_write_u32(file, 0)?; //sir0_pointer_header

        let sir0_pointer_offset = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        wan_write_u32(file, 0)?;

        wan_write_u32(file, 0)?; // magic

        // write meta-frame
        trace!("start of meta frame reference: {}", file.seek(SeekFrom::Current(0))?);
        let meta_frame_references = MetaFrameStore::write(file, &wanimage.meta_frame_store)?;

        trace!("start of the animation offset: {}", file.seek(SeekFrom::Current(0))?);
        let animations_pointer = AnimStore::write(file, &wanimage.anim_store)?;

        file.write_padding(0xAA, 4).unwrap();

        trace!("start of the image offset: {}", file.seek(SeekFrom::Current(0))?);
        let (image_offset, sir0_pointer_images) = ImageStore::write(file, &wanimage)?;

        for pointer in sir0_pointer_images {
            sir0_offsets.push(pointer);
        };


        trace!("start of the palette: {}", file.seek(SeekFrom::Current(0))?);
        let pointer_palette = wanimage.palette.write(file).unwrap();
        //sir0_offsets.push(pointer_palette);

        sir0_offsets.push(pointer_palette);

        trace!("start of the meta_frame reference offset: {}", file.seek(SeekFrom::Current(0))?);
        let meta_frame_reference_offset = file.seek(SeekFrom::Current(0))?;
        for reference in meta_frame_references {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
            file.write_u32_le(reference).unwrap();
        };

        let particule_offset = file.seek(SeekFrom::Current(0))?;
        trace!("start of the particule offset: {}", file.seek(SeekFrom::Current(0))?);
        //HACK: particule offset table parsing is not implement (see the psycommand code of spriteditor)
        file.write_bytes(&wanimage.raw_particule_table).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);

        trace!("start of the animation group reference: {}", file.seek(SeekFrom::Current(0))?);
        let (animation_group_reference_offset, sir0_animation_pointer) = wanimage.anim_store.write_animation_group(file, &animations_pointer).unwrap();
        for pointer in sir0_animation_pointer {
            sir0_offsets.push(pointer);
        };

        //image offset
        let pointer_image_data_pointer_table = file.seek(SeekFrom::Current(0))?;
        trace!("start of the image offset: {}", file.seek(SeekFrom::Current(0))?);
        for offset in image_offset {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
            file.write_u32_le(offset as u32).unwrap();
        };

        // animation header
        let animation_info_offset = file.seek(SeekFrom::Current(0))?;
        trace!("start of the animation header: {}", file.seek(SeekFrom::Current(0))?);
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(meta_frame_reference_offset as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(particule_offset as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(animation_group_reference_offset as u32).unwrap();
        file.write_u16_le((wanimage.anim_store.anim_groups.len()-7) as u16).unwrap(); //HACK:

        // HACK: check what does this mean
        file.write_u32_le(wanimage.unk_1).unwrap();
        wan_write_u32(file, 0)?;
        wan_write_u16(file, 0)?;

        // images header
        trace!("start of the images header: {}", file.seek(SeekFrom::Current(0))?);
        let image_info_offset = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(pointer_image_data_pointer_table as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(pointer_palette as u32).unwrap();
        wan_write_u16(file, 0)?; //HACK: unknow
        file.write_u16_le(if wanimage.is_256_color {
            1
        } else {
            0
        }).unwrap();

        file.write_u16_le(1).unwrap(); //HACK: unknow
        file.write_u16_le(wanimage.image_store.len() as u16).unwrap();

        // wan header
        let wan_header_pos = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(animation_info_offset as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))?);
        file.write_u32_le(image_info_offset as u32).unwrap();
        file.write_u16_le(wanimage.sprite_type.get_id() as u16).unwrap();


        wan_write_u16(file, 0)?;

        file.write_padding(0xAA, 32).unwrap();

        let sir0_offset_pos = file.seek(SeekFrom::Current(0))?;
        // write the sir0 ending

        trace!("start of the sir0 list: {}", file.seek(SeekFrom::Current(0))?);
        Sir0::write_offset_list(file, &sir0_offsets).unwrap();

        file.write_padding(0xAA, 32).unwrap();

        // write the sir0 header
        file.seek(sir0_pointer_header);
        file.write_u32_le(wan_header_pos as u32).unwrap();

        file.seek(sir0_pointer_offset);
        file.write_u32_le(sir0_offset_pos as u32)?;

        file.seek(0);
        Ok(file)
    }*/
}

