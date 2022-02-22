use crate::{encode_image_pixel, ImageBytes, MetaFrame, MetaFrameGroup, Resolution, WanImage};
use anyhow::{bail, Context};
use std::convert::TryInto;

#[derive(Debug)]
struct ImageBuffer {
    buffer: Vec<u8>,
    width: u16,
    height: u16,
}

impl ImageBuffer {
    pub fn new_from_vec(buffer: Vec<u8>, width: u16, height: u16) -> Option<ImageBuffer> {
        if width as usize * height as usize != buffer.len() {
            return None;
        }
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            buffer,
            width,
            height,
        })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u8> {
        if x >= self.width {
            return None;
        }
        self.buffer
            .get(y as usize * self.width as usize + x as usize)
            .copied()
    }

    pub fn get_mut_pixel(&mut self, x: u16, y: u16) -> Option<&mut u8> {
        if x >= self.width {
            return None;
        }
        self.buffer
            .get_mut(y as usize * self.width as usize + x as usize)
    }

    pub fn cut_top(&mut self) -> usize {
        let mut number_of_row_to_cut: u16 = 0;
        for row in self.buffer.chunks_exact(self.width as usize) {
            let mut have_element = false;
            for pixel in row {
                have_element |= *pixel != 0;
            }
            if have_element {
                break;
            } else {
                number_of_row_to_cut += 1;
            }
        }
        let buffer = &self.buffer
            [number_of_row_to_cut as usize * self.width as usize..self.buffer.len()]
            .to_vec();
        self.buffer = buffer.clone();
        self.height -= number_of_row_to_cut;
        number_of_row_to_cut.into()
    }
    }

    pub fn get_chunk_buffer(&self, chunk_size: u16) -> Option<ImageBuffer> {
        if chunk_size == 0 {
            return None;
        };
        let width_nb_ch = (self.width as usize + chunk_size as usize - 1) / chunk_size as usize;
        let height_nb_ch = (self.height as usize + chunk_size as usize - 1) / chunk_size as usize;
        let mut chunk_buffer = Self::new_from_vec(
            vec![0; width_nb_ch * height_nb_ch],
            width_nb_ch as u16,
            height_nb_ch as u16,
        )
        .unwrap();

        for chunk_y in 0..height_nb_ch {
            for chunk_x in 0..width_nb_ch {
                let mut chunk_have_pixel = false;
                for inner_y in 0..chunk_size as usize {
                    for inner_x in 0..chunk_size {
                        let pixel = self.get_pixel(
                            chunk_x as u16 * chunk_size + inner_x as u16,
                            chunk_y as u16 * chunk_size + inner_y as u16,
                        );
                        chunk_have_pixel |= pixel != None && pixel != Some(0);
                    }
                }
                *chunk_buffer
                    .get_mut_pixel(chunk_x as u16, chunk_y as u16)
                    .unwrap() = if chunk_have_pixel { 1 } else { 0 };
            }
        }

        Some(chunk_buffer)
    }
}

#[derive(Debug)]
struct MetaFramePos {
    start: (u16, u16),
    size: Resolution,
}

const MAX_CHUNK_SIZE: u16 = 64;
const MIN_CHUNK_SIZE: u16 = 8;

pub fn insert_frame_in_wanimage(
    image: Vec<u8>,
    width: u16,
    height: u16,
    wanimage: &mut WanImage,
    pal_id: u16,
) -> anyhow::Result<Option<usize>> {
    if height >= 256 {
        bail!("The height of the image is {}, while only image with a height inferior to 256 can be used", height);
    }
    if width >= 512 {
        bail!(
            "The width of the image is {}, while only image with a width less than 512 can be used",
            width
        );
    }
    let position_x = -(width as i32) / 2;
    let mut position_y = -(height as i32) / 2;
    let mut image_buffer = ImageBuffer::new_from_vec(image, width, height).unwrap();

    // find top corner of the image
    position_y += image_buffer.cut_top() as i32;

    //TODO: do the same with the left side

    let list_meta_frame_pos = get_optimal_meta_frames_pos(&image_buffer)?;

    let meta_frames = insert_meta_frame_post_in_wan_image(
        &list_meta_frame_pos,
        wanimage,
        pal_id,
        &image_buffer,
        position_x,
        position_y,
    )?;

    Ok(if !meta_frames.is_empty() {
        let meta_frame_group_id = wanimage.meta_frame_store.meta_frame_groups.len();
        wanimage
            .meta_frame_store
            .meta_frame_groups
            .push(MetaFrameGroup { meta_frames });
        Some(meta_frame_group_id)
    } else {
        None
    })
}

fn get_optimal_meta_frames_pos(image_buffer: &ImageBuffer) -> anyhow::Result<Vec<MetaFramePos>> {
    let chunk_map = image_buffer
        .get_chunk_buffer(MIN_CHUNK_SIZE)
        .context("can't create the chunk map for the smallest chunk size")?;
    let max_chunk_map = chunk_map
        .get_chunk_buffer(MAX_CHUNK_SIZE / MIN_CHUNK_SIZE)
        .context("can't create the chunk map for the biggest chunk size")?;

    //TODO: prefer other form instead of the 64x64 one (like, the 32x32, 16x16, etc)
    let mut list_meta_frame_pos: Vec<MetaFramePos> = Vec::new();
    for max_chunk_y in 0..max_chunk_map.height() {
        for max_chunk_x in 0..max_chunk_map.width() {
            // no panic: iter over width and height, guaranted to be valid
            let pixel_in_max_chunk = max_chunk_map.get_pixel(max_chunk_x, max_chunk_y).unwrap();
            if pixel_in_max_chunk != 0 {
                list_meta_frame_pos.push(MetaFramePos {
                    start: (
                        (max_chunk_x * MAX_CHUNK_SIZE),
                        (max_chunk_y * MAX_CHUNK_SIZE),
                    ),
                    size: Resolution {
                        x: MAX_CHUNK_SIZE as u8,
                        y: MAX_CHUNK_SIZE as u8,
                    },
                })
            }
        }
    }

    Ok(list_meta_frame_pos)
}

fn insert_meta_frame_post_in_wan_image(
    list_meta_frame_pos: &[MetaFramePos],
    wanimage: &mut WanImage,
    pal_id: u16,
    image_buffer: &ImageBuffer,
    position_x: i32,
    position_y: i32,
) -> anyhow::Result<Vec<MetaFrame>> {
    let mut meta_frames = Vec::new();
    let mut image_size_counter = 0;

    for meta_frame_pos in list_meta_frame_pos {
        let mut pixels = Vec::new();
        for in_y in 0..meta_frame_pos.size.y as u16 {
            for in_x in 0..meta_frame_pos.size.x as u16 {
                pixels.push(
                    image_buffer
                        .get_pixel(meta_frame_pos.start.0 + in_x, meta_frame_pos.start.1 + in_y)
                        .unwrap_or(0),
                )
            }
        }
        let image_bytes_id = wanimage.image_store.images.len();
        wanimage.image_store.images.push(ImageBytes {
            mixed_pixels: encode_image_pixel(&pixels, &meta_frame_pos.size)
                .context("failed to encode the input byte. This is likely an internal error")?,
            z_index: 1,
        });
        let offset_y = (position_y + meta_frame_pos.start.1 as i32)
            .try_into()
            .context("The image is too large")?;
        meta_frames.push(MetaFrame {
            unk1: 0,
            unk2: image_size_counter, //TODO: document/rename
            unk3_4: None,
            unk5: false,
            image_index: image_bytes_id,
            offset_y,
            offset_x: (position_x + meta_frame_pos.start.0 as i32)
                .try_into()
                .context("The image is too high")?,
            v_flip: false,
            h_flip: false,
            is_mosaic: false,
            pal_idx: pal_id,
            resolution: meta_frame_pos.size,
        });
        image_size_counter += meta_frame_pos.size.chunk_to_allocate_for_metaframe();
    }

    wanimage.unk_1 = wanimage.unk_1.max(image_size_counter as u32);

    Ok(meta_frames)
}

#[test]
fn quick_imagebuffer_test() {
    let image_buffer = ImageBuffer::new_from_vec(vec![1; 9], 3, 3).unwrap();
    let sub_buffer = image_buffer.get_chunk_buffer(2).unwrap();
    assert_eq!(sub_buffer.width, 2);
    assert_eq!(sub_buffer.height, 2);
    assert_eq!(sub_buffer.buffer, vec![1, 1, 1, 1]);
}
