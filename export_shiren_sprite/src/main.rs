use std::fs::File;

use pmd_wan::shiren::{
    shiren_export_frame, ShirenPalette, ShirenWan,
};
use spritebot_storage::{Animation, Frame, FrameOffset, Sprite};
use vfs::PhysicalFS;

fn main() {
    env_logger::init();
    let shiren_path = "/home/marius/skytemple/shiren/npc/coppa.bin";
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
            for anim_frame in animation.frames.iter() {
                let frame_id = anim_frame.frame_id as usize;
                let frame = shiren_wan.frame_store.frames.get(frame_id).unwrap();

                let (frame_image, frame_image_offset) =
                    shiren_export_frame(frame, &shiren_wan, &shiren_palette).unwrap();

                frames_to_add.push(Frame {
                    duration: anim_frame.frame_duration.try_into().unwrap(),
                    image: frame_image,
                    offsets: FrameOffset {
                        center: (
                            frame_image_offset.0.try_into().unwrap(),
                            frame_image_offset.0.try_into().unwrap(),
                        ),
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
}
