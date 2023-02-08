use std::{path::{PathBuf, Path}, fs::{read_dir, File}, io::BufReader, collections::{HashMap, HashSet}};

use clap::Parser;
use pmd_wan::{shiren::{ShirenWan, ShirenFragment}};

#[derive(Parser, Debug)]
struct Opts {
    decompressed_shiren: PathBuf,
}

// 32 (8×8) is always 0, 252. Only one of this size.
// 512 have 3 possible values: 32*32, 16*64, 64*16
#[derive(Default, Debug)]
struct TestSizeIndices {
    sizes: HashMap<usize, HashSet<u8>>,
    count: HashMap<usize, usize>
}

impl TestSizeIndices {
    fn add(&mut self, fragment: &ShirenFragment, len: usize) {
        self.sizes.entry(len).or_default().insert(fragment.unk2 >> 6);
        *self.count.entry(len).or_default() += 1;
    }
}


fn perform_test(path: &Path, test: &mut TestSizeIndices) {
    println!("{:?}", path);
    let mut file = BufReader::new(File::open(path).unwrap());
    let wan = ShirenWan::new(&mut file).unwrap();
    for frame in &wan.frame_store.frames {
        for fragment in &frame.fragments {
            let fragment_bytes_size = wan.fragment_bytes_store.fragment_bytes[fragment.fragment_bytes_id as usize].bytes.len();
            test.add(&fragment, fragment_bytes_size);
        }
    }
}

fn main() {
    let opts = Opts::parse();
    env_logger::init();

    let mut test = TestSizeIndices::default();

    for directory_name in &["npc", "monster"] {
        for entry in read_dir(opts.decompressed_shiren.join(directory_name)).unwrap().map(|x| x.unwrap()) {
            if entry.file_name() == "shiren_palet.bin" {
                continue
            }
            perform_test(&entry.path(), &mut test);

        }
    }
    println!("{:?}", test);
    
}
