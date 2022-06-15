use criterion::{criterion_group, criterion_main, Criterion};
use pmd_wan::WanImage;
use std::io::Cursor;
use std::fs::File;
use std::io::Read;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut wan_file = File::open("/home/marius/pmd_wan/bulbasaurEU.wan").unwrap();
    let mut wan_data = Vec::new();
    wan_file.read_to_end(&mut wan_data).unwrap();

    c.bench_function("load image", |b| b.iter(|| WanImage::decode_wan(
        Cursor::new(&wan_data)
    ).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);