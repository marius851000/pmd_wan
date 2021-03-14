#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pmd_wan;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let input = Cursor::new(data);
    let _result = pmd_wan::WanImage::new(input);
    /*match result {
        Err(_) => (),
        Ok(valid) => {
            let mut output = Cursor::new(Vec::new());
            pmd_wan::WanImage::create_wan(&valid, &mut output).unwrap(); //TODO: change the API
        }
    }*/
});
