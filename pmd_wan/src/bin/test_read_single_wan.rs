use std::{fs::File, io::{Cursor, Read}};

use pmd_wan::WanImage;

pub fn main() {
    let mut wan_file = File::open("bulbasaurEU.wan").unwrap();
    let mut wan_data = Vec::new();
    wan_file.read_to_end(&mut wan_data).unwrap();
    for _ in 0..10000 {
        WanImage::decode_wan(
            Cursor::new(&wan_data)
        ).unwrap();
    };
}