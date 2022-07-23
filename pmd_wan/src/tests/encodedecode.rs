mod tests {
    use image::{io::Reader as ImageReader, ImageFormat};
    use std::{collections::HashMap, io::Cursor};

    use crate::{insert_fragment_in_wanimage, Animation, AnimationFrame, WanImage};

    #[test]
    fn encode_and_decode_static_wan() {
        let test_image_bytes = include_bytes!("./data/some_image.png");
        let mut reader = ImageReader::new(Cursor::new(test_image_bytes));
        reader.set_format(ImageFormat::Png);
        let image = reader.decode().unwrap().into_rgba8();

        let mut palette = vec![[0, 0, 0, 0]];
        let mut palette_map = HashMap::new();
        let mut image_paletted =
            Vec::with_capacity(image.width() as usize * image.height() as usize);
        for color in image.pixels() {
            if palette_map.get(color).is_none() {
                palette_map.insert(color.clone(), palette.len());
                palette.push(color.0);
            };
            image_paletted.push(*palette_map.get(color).unwrap() as u8);
        }

        let mut wanimage = WanImage::new(crate::SpriteType::PropsUI);
        assert!(palette.len() <= 16);
        palette.resize(16, [0, 0, 0, 255]);
        wanimage.palette.palette = palette.clone();
        let frame_id = insert_fragment_in_wanimage(
            image_paletted,
            image.width() as u16,
            image.height() as u16,
            &mut wanimage,
            0,
        )
        .unwrap()
        .unwrap() as u16;
        let inserted_frame = AnimationFrame {
            duration: 5,
            flag: 0,
            frame_id,
            offset_x: 5,
            offset_y: 5,
            shadow_offset_x: 0,
            shadow_offset_y: 10,
        };
        wanimage.anim_store.anim_groups.push(vec![Animation {
            frames: vec![inserted_frame.clone()],
        }]);

        let mut wan_cursor = Cursor::new(Vec::new());
        wanimage.create_wan(&mut wan_cursor).unwrap();

        let decoded_wanimage = WanImage::decode_wan(&mut wan_cursor).unwrap();
        assert_eq!(decoded_wanimage.palette.palette, palette);
        assert_eq!(
            decoded_wanimage.anim_store.anim_groups[0][0].frames[0],
            inserted_frame
        );
        assert_eq!(
            decoded_wanimage.frames.frames[frame_id as usize]
                .fragments
                .len(),
            4
        );
    }
}
