use crate::{
    encode_fragment_pixels, Fragment, FragmentBytes, FragmentFlip, Frame, GeneralResolution,
    ImageBuffer, OamShape, WanImage,
};
use anyhow::{bail, Context};
use std::convert::TryInto;

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
        let frame_id = wanimage.frame_store.frames.len();
        wanimage.frame_store.frames.push(Frame {
            fragments,
            frame_offset: None,
        });
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
            let fragment_size = OamShape::find_smallest_containing(GeneralResolution::new(
                cut_section.width().into(),
                cut_section.height().into(),
            ))
            .unwrap();

            let buffer_to_write = cut_section.get_fragment(
                0,
                0,
                fragment_size.size().x as u16,
                fragment_size.size().y as u16,
                0,
            );

            let image_bytes_index = wanimage.fragment_bytes_store.fragment_bytes.len();
            wanimage
                .fragment_bytes_store
                .fragment_bytes
                .push(FragmentBytes {
                    mixed_pixels: encode_fragment_pixels(
                        buffer_to_write.buffer(),
                        fragment_size.size(),
                    )
                    .context("failed to encode the input byte. This is an internal error")?,
                    z_index: 1,
                });

            let offset_y = fragment_y.try_into().context("The image is too large")?;
            fragments.push(Fragment {
                unk1: 0,
                unk3_4: None,
                unk5: false,
                fragment_bytes_index: image_bytes_index,
                offset_y,
                offset_x: fragment_x.try_into().context("The image is too high")?,
                flip: FragmentFlip::standard(),
                is_mosaic: false,
                pal_idx: pal_id,
                resolution: fragment_size,
            });
        }
    }

    if fragments.is_empty() {
        return Ok(None);
    }

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
    let frame_id = insert_frame_in_wanimage(vec![1; 36], 6, 6, &mut wanimage, 0)
        .unwrap()
        .unwrap();
    let frame = &wanimage.frame_store.frames[frame_id];
    let fragment = &frame.fragments[0];
    assert_eq!(fragment.resolution, OamShape::new(0, 0).unwrap());
    assert_eq!(fragment.pal_idx, 0);
}

#[test]
fn insert_empty_image_test() {
    let mut wanimage = WanImage::new(crate::SpriteType::PropsUI);
    assert!(insert_frame_in_wanimage(vec![0; 4], 2, 2, &mut wanimage, 0)
        .unwrap()
        .is_none());
}
