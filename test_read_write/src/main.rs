use clap::Parser;
use pmd_cpack::CPack;
use pmd_pkdpx::decompress_px;
use pmd_wan::WanImage;
use std::{
    fs::{read_dir, File},
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Opts {
    decompressed_pmd: PathBuf,
}

fn test_read_reencode<F: Read + Seek>(content: &mut F, add_seven_anim_group: bool, source: &str) {
    println!("trying {}", source);

    let mut buffer_in = Vec::new();
    content.read_to_end(&mut buffer_in).unwrap();
    content.seek(SeekFrom::Start(0)).unwrap();
    //read
    let original_wan = match WanImage::decode_wan(content, add_seven_anim_group) {
        Ok(r) => r,
        Err(e) => {
            let mut f = File::create("./in.bin").unwrap();
            f.write_all(&buffer_in).unwrap();
            panic!("an error occured while reading the original file ({:?}). File written in \"in.bin\"", e);
        }
    };
    //write
    let rewrite_buffer: Vec<u8> = Vec::new();
    let mut rewrite_cursor = Cursor::new(rewrite_buffer);
    original_wan.create_wan(&mut rewrite_cursor).unwrap();
    // copy
    let rewriter_inner = rewrite_cursor.into_inner();
    let buffer_out = rewriter_inner.clone();
    let mut rewrite_cursor = Cursor::new(rewriter_inner);
    //re-read
    rewrite_cursor.seek(SeekFrom::Start(0)).unwrap();
    let reread_wan = WanImage::decode_wan(rewrite_cursor, add_seven_anim_group);

    let reread_wan = match reread_wan {
        Ok(r) => Some(r),
        Err(err) => {
            println!("the error while reading was {:?}", err);
            None
        }
    };
    if reread_wan == None || reread_wan.unwrap() != original_wan {
        // write the in.bin and out.bin file
        let mut in_file = File::create("in.bin").unwrap();
        in_file.write_all(&buffer_in).unwrap();

        let mut out_file = File::create("out.bin").unwrap();
        out_file.write_all(&buffer_out).unwrap();

        //let reread_wan = reread_wan.unwrap();

        panic!(
            "failed to correctly read, write and re-read the written file for {}",
            source
        );
    }
}

fn main() {
    let opts = Opts::parse();
    println!("trying to decode and re-encode byte perfect all sprites in the decompressed PMD explorers rom at {:?}", opts.decompressed_pmd);

    println!("trying this on objects");

    env_logger::init();

    for (monster_file_name, decompress, add_seven_anim_group) in [
        ("m_ground.bin", false, true),
        ("monster.bin", true, false),
        ("m_attack.bin", true, false),
    ] {
        let path = opts
            .decompressed_pmd
            .join("MONSTER")
            .join(monster_file_name);
        let cpack_file = File::open(&path).unwrap();
        let cpack = CPack::new_from_file(cpack_file).unwrap();
        for sub_file_id in 0..cpack.len() {
            let mut sub_file = cpack.get_file(sub_file_id).unwrap();
            let sub_file_vec = if decompress {
                decompress_px(sub_file).unwrap()
            } else {
                let mut buffer = Vec::new();
                sub_file.read_to_end(&mut buffer).unwrap();
                buffer
            };
            let mut cursor = Cursor::new(sub_file_vec);
            test_read_reencode(
                &mut cursor,
                add_seven_anim_group,
                &format!("{:?} sub file n°{}", path, sub_file_id),
            );
        }
    }

    for entry in read_dir(opts.decompressed_pmd.join("GROUND")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut f = File::open(&path).unwrap();
        println!("{:?}", path);
        if path.extension().unwrap() == "wan" {
            test_read_reencode(&mut f, false, &path.to_string_lossy());
        }
    }
    //test_read_reencode(&PathBuf::from("/home/marius/pmdeu/GROUND/d01p11b2.wan"));
}
