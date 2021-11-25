use clap::Parser;
use pmd_wan::{WanImage};
use std::{
    fs::{read_dir, File},
    io::{Cursor, Read, Seek, SeekFrom},
    path::{PathBuf},
};

#[derive(Parser, Debug)]
struct Opts {
    decompressed_pmd: PathBuf,
}

fn test_read_reencode<F: Read + Seek>(content: &mut F, source: &str) {
    //read
    let original_wan = WanImage::new(content).unwrap();
    //write
    let rewrite_buffer: Vec<u8> = Vec::new();
    let mut rewrite_cursor = Cursor::new(rewrite_buffer);
    original_wan.create_wan(&mut rewrite_cursor).unwrap();
    //re-read
    rewrite_cursor.seek(SeekFrom::Start(0)).unwrap();
    let reread_wan = WanImage::new(rewrite_cursor).unwrap();
    if original_wan != reread_wan {
        panic!("failed to correctly read, write and re-read the written file for {}", source);
    }
}

fn main() {
    let opts = Opts::parse();
    println!("trying to decode and re-encode byte perfect all sprites in the decompressed PMD explorers rom at {:?}", opts.decompressed_pmd);

    println!("trying this on objects");

    for entry in read_dir(opts.decompressed_pmd.join("GROUND")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut f = File::open(&path).unwrap();
        if path.extension().unwrap() == "wan" {
            test_read_reencode(&mut f, &path.to_string_lossy());
        }
    }
    //test_read_reencode(&PathBuf::from("/home/marius/pmdeu/GROUND/d01p11b2.wan"));
}
