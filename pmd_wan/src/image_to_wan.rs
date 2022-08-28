use crate::{
    encode_fragment_pixels, Fragment, FragmentFlip, FragmentResolution, Frame, ImageBytes, WanImage,
};
use anyhow::{bail, Context};
use std::convert::TryInto;

/// Images with no pixel are valid, but it is guarantted that width*height == buffer.len()
#[derive(Debug, PartialEq, Eq)]
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

    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    pub fn have_pixel(&self) -> bool {
        self.width != 0 || self.height != 0
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u8> {
        if x >= self.width {
            return None;
        }
        self.buffer
            .get(y as usize * self.width as usize + x as usize)
            .copied()
    }

    pub fn cut_top(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_line_to_cut: u16 = 0;
        for row in self.buffer.chunks_exact(self.width as usize) {
            let mut have_element = false;
            for pixel in row {
                have_element |= *pixel != 0;
            }
            if have_element {
                break;
            } else {
                number_of_line_to_cut += 1;
            }
        }
        let buffer = &self.buffer
            [number_of_line_to_cut as usize * self.width as usize..self.buffer.len()]
            .to_vec();
        self.buffer = buffer.clone();
        self.height -= number_of_line_to_cut;
        number_of_line_to_cut.into()
    }

    pub fn cut_bottom(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_cut_line = 0;
        'main: for line_nb in (0..self.height).rev() {
            for pixel_nb in (line_nb * self.width)..((line_nb + 1) * self.width) {
                //no panic: pixel_nb should always be in the appropriate range
                if self.buffer[pixel_nb as usize] != 0 {
                    break 'main;
                };
            }
            number_of_cut_line += 1;
            self.height -= 1;
            self.buffer
                .truncate(self.height as usize * self.width as usize);
        }
        number_of_cut_line
    }

    pub fn cut_right(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_cut_row = 0;
        'main: for _row in (0..self.width).rev() {
            for line in 0..(self.height as usize) {
                let pixel_id = self.width as usize * line + self.width as usize - 1;
                if self.buffer[pixel_id] != 0 {
                    break 'main;
                };
            }
            for line in (0..(self.height as usize)).rev() {
                let pixel_to_remove = self.width as usize * line + self.width as usize - 1;
                self.buffer.remove(pixel_to_remove);
            }
            self.width -= 1;
            number_of_cut_row += 1;
        }
        number_of_cut_row
    }

    pub fn cut_left(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_cut_row = 0;
        'main: for _row in (0..self.width).rev() {
            for line in 0..(self.height as usize) {
                let pixel_id = self.width as usize * line;
                if self.buffer[pixel_id] != 0 {
                    break 'main;
                };
            }
            for line in (0..self.height).rev() {
                let pixel_to_remove = self.width as usize * line as usize;
                self.buffer.remove(pixel_to_remove);
            }
            self.width -= 1;
            number_of_cut_row += 1;
        }
        number_of_cut_row
    }

    pub fn get_fragment(
        &self,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
        default: u8,
    ) -> ImageBuffer {
        let mut buffer = Vec::new();
        for y in start_y..start_y + height {
            for x in start_x..start_x + width {
                buffer.push(self.get_pixel(x, y).unwrap_or(default));
            }
        }
        ImageBuffer::new_from_vec(buffer, width, height).unwrap()
    }
}

pub fn insert_fragment_in_wanimage(
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
    let position_y = -(height as i32) / 2;
    let image_buffer = ImageBuffer::new_from_vec(image, width, height)
        .context("The input image don't correspond to the dimension of it")?;

    let fragments = if let Some(fragments) =
        insert_fragment_pos_in_wan_image(wanimage, pal_id, &image_buffer, position_x, position_y)?
    {
        fragments
    } else {
        return Ok(None);
    };

    Ok(if !fragments.is_empty() {
        let frame_id = wanimage.frames.frames.len();
        wanimage.frames.frames.push(Frame { fragments });
        Some(frame_id)
    } else {
        None
    })
}

fn insert_fragment_pos_in_wan_image(
    wanimage: &mut WanImage,
    pal_id: u16,
    image_buffer: &ImageBuffer,
    upper_image_x: i32,
    upper_image_y: i32,
) -> anyhow::Result<Option<Vec<Fragment>>> {
    let mut fragments = Vec::new();
    let mut image_alloc_counter = 0;

    // Chunk the image into 64x64 group, the max meta frame size
    const MAX_META_FRAME_SIZE: u16 = 64;
    for fragment_segment_x in
        0..(image_buffer.width() + MAX_META_FRAME_SIZE - 1) / MAX_META_FRAME_SIZE
    {
        for fragment_segment_y in
            0..(image_buffer.height() + MAX_META_FRAME_SIZE - 1) / MAX_META_FRAME_SIZE
        {
            let mut fragment_x =
                upper_image_x + MAX_META_FRAME_SIZE as i32 * fragment_segment_x as i32;
            let mut fragment_y =
                upper_image_y + MAX_META_FRAME_SIZE as i32 * fragment_segment_y as i32;

            let mut cut_section = image_buffer.get_fragment(
                MAX_META_FRAME_SIZE * fragment_segment_x,
                MAX_META_FRAME_SIZE * fragment_segment_y,
                MAX_META_FRAME_SIZE,
                MAX_META_FRAME_SIZE,
                0,
            );
            fragment_y += cut_section.cut_top() as i32;
            cut_section.cut_bottom();
            fragment_x += cut_section.cut_left() as i32;
            cut_section.cut_right();

            if !cut_section.have_pixel() {
                continue;
            }

            //no panic: resolution should always be less than 64x64, and be an already valid resolution, to which it can fall back if no smaller images are avalaible
            let fragment_size = FragmentResolution::find_smaller_containing(FragmentResolution {
                x: cut_section.width() as u8,
                y: cut_section.height() as u8,
            })
            .unwrap();

            let buffer_to_write =
                cut_section.get_fragment(0, 0, fragment_size.x as u16, fragment_size.y as u16, 0);

            let image_bytes_id = wanimage.image_store.images.len();
            wanimage.image_store.images.push(ImageBytes {
                mixed_pixels: encode_fragment_pixels(buffer_to_write.buffer(), &fragment_size)
                    .context("failed to encode the input byte. This is an internal error")?,
                z_index: 1,
            });

            let offset_y = fragment_y.try_into().context("The image is too large")?;
            fragments.push(Fragment {
                unk1: 0,
                image_alloc_counter,
                unk3_4: None,
                unk5: false,
                image_index: image_bytes_id,
                offset_y,
                offset_x: fragment_x.try_into().context("The image is too high")?,
                flip: FragmentFlip::Standard,
                is_mosaic: false,
                pal_idx: pal_id,
                resolution: fragment_size,
            });
            image_alloc_counter += fragment_size.chunk_to_allocate_for_metaframe();
        }
    }

    if fragments.is_empty() {
        return Ok(None);
    }

    wanimage.size_to_allocate_for_all_metaframe = wanimage
        .size_to_allocate_for_all_metaframe
        .max(image_alloc_counter as u32);

    Ok(Some(fragments))
}

#[test]
fn imagebuffer_cut_test() {
    // (image_buffer, x_src, y_src, target_buffer, x_target, y_target, cut_top, cut_bottom, cut_left, cut_right)
    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    let tests_to_perform: [(Vec<u8>, u16, u16, Vec<u8>, u16, u16, usize, usize, usize, usize); 2] = [
        (
            vec![
                0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
                0, 0, 1, 1, 1, 0,
                0, 0, 0, 0, 1, 0,
                0, 0, 0, 0, 0, 0
            ], 6, 5,
            vec![
                1, 1, 1,
                0, 0, 1
            ], 3, 2,
            2, 1, 2, 1
        ),
        (
            vec![
                1, 1, 1,
                1, 1, 1,
                1, 1, 1
            ], 3, 3,
            vec![
                1, 1, 1,
                1, 1, 1,
                1, 1, 1
            ], 3, 3,
            0, 0, 0, 0
        )
    ];
    for (
        buffer_src,
        x_src,
        y_src,
        buffer_target,
        x_target,
        y_target,
        cut_top_px,
        cut_bottom_px,
        cut_left_px,
        cut_right_px,
    ) in tests_to_perform
    {
        let mut image_src = ImageBuffer::new_from_vec(buffer_src, x_src, y_src).unwrap();
        let image_target = ImageBuffer::new_from_vec(buffer_target, x_target, y_target).unwrap();
        assert_eq!(cut_top_px, image_src.cut_top());
        assert_eq!(cut_bottom_px, image_src.cut_bottom());
        assert_eq!(cut_left_px, image_src.cut_left());
        assert_eq!(cut_right_px, image_src.cut_right());
        assert_eq!(image_src, image_target);
    }

    //test with all 0 pixels
    let mut image_src = ImageBuffer::new_from_vec(vec![0; 4], 2, 2).unwrap();
    image_src.cut_top();
    image_src.cut_bottom();
    image_src.cut_right();
    image_src.cut_right();
    assert_eq!(image_src, ImageBuffer::new_from_vec(vec![], 0, 0).unwrap());
}

#[test]
fn get_image_fragment_test() {
    let image_buffer = ImageBuffer::new_from_vec(vec![1, 1, 0, 1, 2, 3, 1, 3, 0], 3, 3).unwrap();
    let fragment = image_buffer.get_fragment(1, 1, 3, 2, 0);
    assert_eq!(
        fragment,
        ImageBuffer::new_from_vec(vec![2, 3, 0, 3, 0, 0], 3, 2).unwrap()
    );
}

#[test]
fn insert_frame_flat_test() {
    let mut wanimage = WanImage::new(crate::SpriteType::PropsUI);
    wanimage.palette.palette.push([255, 255, 255, 128]);
    let frame_id = insert_fragment_in_wanimage(vec![1; 36], 6, 6, &mut wanimage, 0)
        .unwrap()
        .unwrap();
    let frame = &wanimage.frames.frames[frame_id];
    let fragment = &frame.fragments[0];
    assert_eq!(fragment.resolution, FragmentResolution { x: 8, y: 8 });
    assert_eq!(fragment.pal_idx, 0);
}

#[test]
fn insert_empty_image_test() {
    let mut wanimage = WanImage::new(crate::SpriteType::PropsUI);
    assert!(
        insert_fragment_in_wanimage(vec![0; 4], 2, 2, &mut wanimage, 0)
            .unwrap()
            .is_none()
    );
}
