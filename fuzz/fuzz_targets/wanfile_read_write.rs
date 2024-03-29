#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pmd_wan;
use std::io::Cursor;
use std::io::{Seek, SeekFrom};

//TODO: create wan from png or something (and then decode)
fuzz_target!(|data: &[u8]| {
    let input = Cursor::new(data);
    let decoded = pmd_wan::WanImage::decode_wan(input);
    match decoded {
        Err(_) => (),
        Ok(mut valid) => {
            valid.animation_store.copied_on_previous = None;
            let mut reread_file = Cursor::new(Vec::new());
            valid.create_wan(&mut reread_file).unwrap();
            reread_file.seek(SeekFrom::Start(0)).unwrap();
            let mut reread_wan = pmd_wan::WanImage::decode_wan(reread_file).unwrap();
            reread_wan.animation_store.copied_on_previous = None;
            //TODO: I don’t have time for those details
            if valid.animation_store.anim_groups.len() != 0 && valid.animation_store.anim_groups.iter().map(|x| x.len()).min() != Some(0) {
                assert_eq!(valid, reread_wan);
            }
        }
    }
});
