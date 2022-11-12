mod tests {
    use image::{io::Reader as ImageReader, ImageFormat};
    use std::io::Cursor;

    use crate::{
        image_tool::{image_to_paletted_bytes, ImageToPaletteBytesData},
        insert_frame_in_wanimage, Animation, AnimationFrame, WanImage,
    };

    #[test]
    fn encode_and_decode_static_wan() {
        let test_image_bytes = include_bytes!("./data/some_image.png");
        let mut reader = ImageReader::new(Cursor::new(test_image_bytes));
        reader.set_format(ImageFormat::Png);
        let image = reader.decode().unwrap().into_rgba8();

        let mut palette_data = ImageToPaletteBytesData::default();
        let image_paletted = image_to_paletted_bytes(&mut palette_data, &image).unwrap();

        let mut wanimage = WanImage::new(crate::SpriteType::PropsUI);
        assert!(palette_data.ordered.len() <= 16);
        palette_data.ordered.resize(16, [0, 0, 0, 0]);
        wanimage.palette.palette = palette_data.ordered.clone();
        let frame_id = insert_frame_in_wanimage(
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
        assert_eq!(decoded_wanimage.palette.palette, palette_data.ordered);
        assert_eq!(
            decoded_wanimage.anim_store.anim_groups[0][0].frames[0],
            inserted_frame
        );
        assert_eq!(
            decoded_wanimage.frame_store.frames[frame_id as usize]
                .fragments
                .len(),
            3
        );
    }
}
