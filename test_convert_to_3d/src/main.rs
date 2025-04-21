use std::{
    fs::{read_dir, File},
    io::{Seek, SeekFrom},
    path::PathBuf,
};

use pmd_wan::{SpriteType, WanError, WanImage};

fn main() {
    let in_folder = PathBuf::from("/home/marius/eoseu/GROUND/");
    let out_folder =
        PathBuf::from("/home/marius/more3dcot/c-of-time/build_ressources/fs_patch_temp/GROUND/");
    for entry in read_dir(&in_folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut f = File::open(&path).unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        println!("{:?}", path);
        if path.extension().unwrap() == "wan" {
            let mut original_wan = match WanImage::decode_wan(&mut f) {
                Ok(x) => x,
                Err(WanError::FragmentBytesIDPointBackButFirstFragment) => {
                    println!("Skipping {}", path.file_name().unwrap().to_str().unwrap());
                    continue;
                }
                Err(e) => {
                    panic!("an error occured while reading the original file ({:?}). File written in \"in.bin\"", e);
                }
            };

            let out_path = out_folder.join(path.file_name().unwrap());
            let mut out_file = File::create(&out_path).unwrap();

            original_wan.sprite_type = SpriteType::Engine3D;

            original_wan.create_wan(&mut out_file).unwrap();
        }
    }
}
