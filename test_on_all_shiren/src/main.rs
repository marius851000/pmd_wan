use std::{
    collections::{HashMap, HashSet},
    fs::{read_dir, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use clap::Parser;
use pmd_wan::{shiren::{ShirenFragment, ShirenWan, shiren_export_fragment, ShirenPalette}, get_bit_u16};

#[derive(Parser, Debug)]
struct Opts {
    decompressed_shiren: PathBuf,
}

// 32 (8×8) is always 0, 252. Only one of this size.
// 512 have 3 possible values: 32*32, 16*64, 64*16
#[derive(Default, Debug)]
struct TestSizeIndices {
    sizes: HashMap<usize, HashSet<u8>>,
    recognized_initialized: bool,
    recognized_value: [Option<bool>; 16], // that’s only for 32x32 images
    count: HashMap<usize, usize>,
}

impl TestSizeIndices {
    fn add(&mut self, fragment: &ShirenFragment, len: usize) {
        self.sizes
            .entry(len)
            .or_default()
            .insert(fragment.oam_shape.size_indice() as u8);
        *self.count.entry(len).or_default() += 1;
        if true /*len == 128*/ {
            for pos in 0..16 {
                let bit = get_bit_u16(fragment.unk1, pos as u16).unwrap();
                if let Some(current_value) = self.recognized_value[pos] {
                    if bit != current_value {
                        self.recognized_value[pos] = None;
                    }
                }
                if self.recognized_initialized == false {
                    self.recognized_value[pos] = Some(bit);
                }
            }
            self.recognized_initialized = true;
        }
    }
}

fn perform_test(path: &Path, test: &mut TestSizeIndices) {
    println!("{:?}", path);

    let shiren_palette_path = "/home/marius/skytemple/shiren/monster/shiren_palet.bin";
    let mut shiren_palette_file = File::open(shiren_palette_path).unwrap();
    let shiren_palette = ShirenPalette::new(&mut shiren_palette_file).unwrap();

    let mut file = BufReader::new(File::open(path).unwrap());
    let wan = ShirenWan::new(&mut file).unwrap();
    let mut fragment_uid = 0;
    for frame in &wan.frame_store.frames {
        let mut _last_fragment = None;
        let mut _previous_fragment = None;
        for fragment in &frame.fragments {
            if let Some(fragment_bytes_id) = fragment.fragment_bytes_id {
                
                let fragment_bytes_size = wan.fragment_bytes_store.fragment_bytes
                    [fragment_bytes_id as usize]
                    .bytes
                    .len();
                /*assert_eq!(fragment.size_indice_y, match fragment_bytes_size {
                    32 => 0,
                    128 => 1,
                    256 => 2,
                    512 => 3,
                    _ => todo!(),
                });*/
                
                if true {
                    let export_file_name = format!("testimage/{}-{}-{}-{}-{}.png", fragment_bytes_size, fragment.oam_shape.shape_indice(), fragment.oam_shape.size_indice(), fragment_uid, path.file_name().unwrap().to_string_lossy());
                    let image = shiren_export_fragment(fragment,  &wan.fragment_bytes_store.fragment_bytes[fragment_bytes_id as usize], &shiren_palette).unwrap();
                    image.save(&export_file_name).unwrap();
                    fragment_uid += 1;
                }
            }

            test.add(&fragment, 0);


            /*if let Some(previous_fragment) = previous_fragment {
                test.add(previous_fragment, 0);
            }*/
            _previous_fragment = Some(fragment);
            _last_fragment = Some(fragment);
        }

        /*if let Some(last_fragment) = last_fragment {
            test.add(&last_fragment, 0);
        }*/
    }
    //println!("animation count: {}", wan.animation_store.animations.len());
}

fn main() {
    let opts = Opts::parse();
    env_logger::init();

    let mut test = TestSizeIndices::default();

    for directory_name in &["npc", "monster"] {
        for entry in read_dir(opts.decompressed_shiren.join(directory_name))
            .unwrap()
            .map(|x| x.unwrap())
        {
            if entry.file_name() == "shiren_palet.bin" {
                continue;
            }
            perform_test(&entry.path(), &mut test);
        }
    }
    println!("{:?}", test);
}
