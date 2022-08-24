use std::{fs::File, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use image::GenericImageView;
use pmd_wan::{
    find_fragments_in_images,
    image_tool::{image_to_paletted_bytes, ImageToPaletteBytesData},
    GeneralResolution,
};

const TILE_WIDTH: u32 = 82;
const TILE_HEIGTH: u32 = 80;
const TILE_NB_X: u32 = 15;
const TILE_NB_Y: u32 = 15;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut img_file = BufReader::new(
        File::open("/home/marius/Sync/programming_project/pmd_wan/reshiram.png").unwrap(),
    );
    let img_generic = image::load(&mut img_file, image::ImageFormat::Png).unwrap();
    let img = img_generic.as_rgba8().unwrap();

    let mut palette_data = ImageToPaletteBytesData::default();
    let mut tiles = Vec::new();
    for tile_x in 0..TILE_NB_X {
        for tile_y in 0..TILE_NB_Y {
            let tile_start_x = tile_x * TILE_WIDTH;
            let tile_start_y = tile_y * TILE_HEIGTH;
            let tile = img
                .view(tile_start_x, tile_start_y, TILE_WIDTH, TILE_HEIGTH)
                .to_image();
            let tile_paletted = image_to_paletted_bytes(&mut palette_data, &tile).unwrap();

            tiles.push(tile_paletted);
        }
    }

    let mut find_fragments_entry = Vec::new();
    for t in &tiles {
        find_fragments_entry.push((
            t.as_slice(),
            GeneralResolution::new(TILE_WIDTH, TILE_HEIGTH),
        ));
    }

    c.bench_function("find fragments", |b| {
        b.iter(|| find_fragments_in_images(&find_fragments_entry).unwrap());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
