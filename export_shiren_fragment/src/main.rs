use std::fs::File;

use pmd_wan::shiren::{ShirenWan, ShirenPalette, shiren_export_fragment};

fn main() {
    env_logger::init();
    let shiren_path = "/home/marius/skytemple/shiren/monster/shiren.bin";
    let shiren_palette_path = "/home/marius/skytemple/shiren/monster/shiren_palet.bin";
    let mut shiren_file = File::open(shiren_path).unwrap();
    let shiren_wan = ShirenWan::new(&mut shiren_file).unwrap();
    let mut shiren_palette_file = File::open(shiren_palette_path).unwrap();
    let shiren_palette = ShirenPalette::new(&mut shiren_palette_file).unwrap();
    
    for frame in &shiren_wan.frame_store.frames {
        for fragment in &frame.fragments {
            let fragment_bytes = shiren_wan.fragment_bytes_store.fragment_bytes.get(fragment.fragment_bytes_id as usize).unwrap();
            if fragment_bytes.bytes.len() == 512 {
                let image = shiren_export_fragment(fragment, fragment_bytes, &shiren_palette).unwrap();
                print!("{:?}\n{}\n", fragment, fragment_bytes.bytes.len());
                image.save(format!("test-{}-{}-{}-{}.png", fragment.unk1, fragment.unk2, fragment.unk4, fragment.unk5)).unwrap();
            }
        }
    }
}