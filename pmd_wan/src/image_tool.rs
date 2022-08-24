use std::{collections::HashMap, convert::TryInto};

use image::{GenericImageView, Rgba};

pub struct ImageToPaletteBytesData {
    pub map: HashMap<[u8; 4], u8>,
    pub ordered: Vec<[u8; 4]>,
}

impl Default for ImageToPaletteBytesData {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert([0, 0, 0, 0], 0);
        Self {
            map,
            ordered: vec![[0, 0, 0, 0]],
        }
    }
}

impl ImageToPaletteBytesData {
    pub fn get_or_insert_id_for_color(&mut self, color: Rgba<u8>) -> Option<u8> {
        if let Some(value) = self.map.get(&color.0) {
            return Some(*value);
        };
        let number = match (self.map.len() + 1).try_into() {
            Err(_) => return None,
            Ok(nb) => nb,
        };
        self.map.insert(color.0, number);
        self.ordered.push(color.0);
        Some(self.map.len() as u8)
    }
}

/// Transform an [`ImageBuffer`] to a list of bytes (its pixels from top left to bottom right, line by line).
/// The [`ImageToPaletteBytesData`] can be used on multiple image to make sure the same color have the same palette id.
/// None is returned if the palette have reach its limit of 255 different color.
pub fn image_to_paletted_bytes<I: GenericImageView<Pixel = Rgba<u8>>>(
    palette_data: &mut ImageToPaletteBytesData,
    img: &I,
) -> Option<Vec<u8>> {
    let mut result = Vec::with_capacity(img.width() as usize * img.height() as usize);
    for (_, _, color) in img.pixels() {
        let mut color = color;
        if color.0[3] != 255 {
            color = Rgba::from([0, 0, 0, 0])
        }
        result.push(palette_data.get_or_insert_id_for_color(color)?);
    }
    Some(result)
}
