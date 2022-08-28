use std::{fs::File, io::BufReader};

use image::GenericImageView;
use pmd_wan::{
    create_wan_from_multiple_images,
    image_tool::{image_to_paletted_bytes, ImageToPaletteBytesData},
    GeneralResolution, SpriteType,
};

const TILE_WIDTH: u32 = 82;
const TILE_HEIGTH: u32 = 80;
const TILE_NB_X: u32 = 15;
const TILE_NB_Y: u32 = 15;

pub fn main() {
    let mut img_file = BufReader::new(
        File::open("/home/marius/Sync/programming_project/pmd_wan/reshiram.png").unwrap(),
    );
    let img_generic = image::load(&mut img_file, image::ImageFormat::Png).unwrap();
    let img = img_generic.as_rgba8().unwrap();

    let mut palette_data = ImageToPaletteBytesData::default();
    let mut tiles = Vec::new();
    let mut nb = 0;
    for tile_y in 0..TILE_NB_Y {
        for tile_x in 0..TILE_NB_X {
            let tile_start_x = tile_x * TILE_WIDTH;
            let tile_start_y = tile_y * TILE_HEIGTH;
            println!("nb: {}, x: {}, y: {}", nb, tile_x, tile_y);
            nb += 1;
            let tile = img
                .view(tile_start_x, tile_start_y, TILE_WIDTH, TILE_HEIGTH)
                .to_image();
            let tile_paletted = image_to_paletted_bytes(&mut palette_data, &tile).unwrap();

            tiles.push(tile_paletted);
        }
    }

    let mut images = Vec::new();
    for t in &tiles {
        images.push((
            t.as_slice(),
            GeneralResolution::new(TILE_WIDTH, TILE_HEIGTH),
        ));
    }

    let mut wan_image = create_wan_from_multiple_images(&images, SpriteType::Chara).unwrap();
    wan_image.palette.palette = palette_data.ordered.clone();
}
