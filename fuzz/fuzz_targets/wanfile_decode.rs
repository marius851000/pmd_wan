#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate pmd_wan;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let input = Cursor::new(data);
    let _ = pmd_wan::WanImage::decode_wan(input);
});