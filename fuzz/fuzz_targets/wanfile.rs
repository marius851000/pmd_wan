#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate pmd_wan;
use std::io::Cursor;
fuzz_target!(|data: &[u8]| {
    let input = Cursor::new(data);
    let _result = pmd_wan::WanImage::new(input);
    /*match result {
        Ok(data) => for image in data.image_store.images {
            if image.img.width() > 3 && image.img.height() > 3 {
                image.img.save("./tmp.png").unwrap();
            };
        },
        Err(_) => (),
    }*/
});
