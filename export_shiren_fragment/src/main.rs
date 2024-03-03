use std::fs::File;

use pmd_wan::shiren::{shiren_export_fragment, ShirenFragment, ShirenPalette, ShirenWan};
use spritebot_storage::{Animation, Frame, FrameOffset, Sprite};
use image::imageops::flip_horizontal;
use vfs::PhysicalFS;

fn main() {
    env_logger::init();
    let shiren_path = "/home/marius/skytemple/shiren/monster/shiren.bin";
    let shiren_palette_path = "/home/marius/skytemple/shiren/monster/shiren_palet.bin";
    let mut shiren_file = File::open(shiren_path).unwrap();
    let shiren_wan = ShirenWan::new(&mut shiren_file).unwrap();
    let mut shiren_palette_file = File::open(shiren_palette_path).unwrap();
    let shiren_palette = ShirenPalette::new(&mut shiren_palette_file).unwrap();

    let mut sprite_export = Sprite::new_empty(0);
    for (animation_group_count, animation_group) in
        shiren_wan.animation_store.animations.iter().enumerate()
    {
        let mut images = Vec::new();

        for animation in animation_group.iter() {
            let mut frames_to_add = Vec::new();
            for frame in animation.frames.iter() {
                let mut fragment_to_use = None;
                let frame_id = frame.frame_id as usize;
                for fragment in shiren_wan.frame_store.frames[frame_id].fragments.iter() {
                    if fragment.fragment_bytes_id.is_some() {
                        fragment_to_use = Some(fragment);
                    }
                }

                if fragment_to_use.is_none() {
                    //TODO:
                    continue;
                }
                let fragment_to_use = fragment_to_use.unwrap();

                let fragment_bytes = shiren_wan
                    .fragment_bytes_store
                    .fragment_bytes
                    .get(fragment_to_use.fragment_bytes_id.unwrap() as usize)
                    .unwrap();

                if fragment_bytes.bytes.len() != 512 {
                    println!("{:?}", fragment_to_use);
                    println!("frame {} does not have the 32 by 32 resolution", frame_id);
                }

                let mut image = shiren_export_fragment(fragment_to_use, fragment_bytes, &shiren_palette).unwrap();
                if fragment_to_use.is_h_flip {
                    image = flip_horizontal(&image);
                }
                frames_to_add.push(Frame {
                    duration: frame.frame_duration.try_into().unwrap(),
                    image,
                    offsets: FrameOffset {
                        center: (0, 0),
                        hand_left: (0, 0),
                        hand_right: (0, 0),
                        head: (1, 0),
                        shadow: (0, 0),
                    },
                })
            }
            images.push(frames_to_add);
        }

        sprite_export.animations.push(Animation {
            name: format!("animation{}", animation_group_count),
            index: animation_group_count.try_into().unwrap(),
            rush_frame: None,
            hit_frame: None,
            return_frame: None,
            images,
        });
    }

    let mut dest_fs = PhysicalFS::new("./test");

    sprite_export.write_to_folder(&mut dest_fs).unwrap()

    /*for (frame_id, frame) in shiren_wan.frame_store.frames.iter().enumerate() {
        for (fragment_id, fragment) in frame.fragments.iter().enumerate() {
            if let Some(fragment_bytes_id) = fragment.fragment_bytes_id {
                let fragment_bytes = shiren_wan
                    .fragment_bytes_store
                    .fragment_bytes
                    .get(fragment_bytes_id as usize)
                    .unwrap();
                if fragment_bytes.bytes.len() == 512 {
                    let image =
                        shiren_export_fragment(fragment, fragment_bytes, &shiren_palette).unwrap();
                    print!("{:?}\n{}\n", fragment, fragment_bytes.bytes.len());
                    image
                        .save(format!(
                            "test-{}-{}-{}-{}-{}-{}.png",
                            frame_id,
                            fragment_id,
                            fragment.unk1,
                            fragment.unk2,
                            fragment.unk4,
                            fragment.unk5
                        ))
                        .unwrap();
                } else {
                    println!("Skipped an image with fragment_bytes.bytes.len is != 512");
                }
            } else {
                //TODO: do something
            }
        }
    }*/
}
