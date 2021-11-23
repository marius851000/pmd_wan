use clap::Parser;
use image::ImageFormat;
use pmd_wan::WanImage;
use std::{
    fs::{read_dir, File},
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
struct Opts {
    decompressed_pmd: PathBuf,
}

fn test_read_reencode(path: &Path) {
    println!("trying {:?}", path);
    let mut f = File::open(&path).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let buffer_in = buffer.clone();
    let input = Cursor::new(buffer);
    let out_buffer = Vec::new();
    let mut output = Cursor::new(out_buffer);
    let wan_image = WanImage::new(input).unwrap();
    wan_image.create_wan(&mut output).unwrap();
    output.seek(SeekFrom::Start(0)).unwrap();
    let out_buffer = output.into_inner();
    let out_buffer_clone = out_buffer.clone();
    let wan_image_redecoded = WanImage::new(Cursor::new(out_buffer_clone)).unwrap();
    if wan_image != wan_image_redecoded {
        let mut i = File::create("./in.bin").unwrap();
        let mut o = File::create("./out.bin").unwrap();
        i.write_all(&buffer_in).unwrap();
        o.write_all(&out_buffer).unwrap();
        if wan_image.image_store.images[0].pixels
            != wan_image_redecoded.image_store.images[0].pixels
        {
            panic!("good");
        }
        panic!("the input and the output of the file at {:?} are different. Content written in \"in.bin\" and \"out.bin\"", path);
    };
}

fn main() {
    let opts = Opts::parse();
    println!("trying to decode and re-encode byte perfect all sprites in the decompressed PMD explorers rom at {:?}", opts.decompressed_pmd);

    println!("trying this on objects");

    for entry in read_dir(opts.decompressed_pmd.join("GROUND")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "wan" {
            test_read_reencode(&path);
        }
    }
}
