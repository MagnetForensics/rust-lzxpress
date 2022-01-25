#![feature(test)]

extern crate test;
extern crate lzxpress;

use test::Bencher;

#[bench]
#[cfg(windows)]
fn bench_lznt1_decompress_rtl(b: &mut Bencher) {
    let compression_format_lznt1 = 0x0002 as u16;
    let compression_engine_standard = 0x0000 as u16;
    use ntapi::ntrtl::{RtlDecompressBuffer};

    let compressed_data = include_bytes!("../tests/block1.compressed.bin");
    let mut mutable_compressed_data = compressed_data.to_vec();

    b.iter(|| {
        let _ret = unsafe {
            let psrc = mutable_compressed_data.as_mut_ptr();
            let srclen = mutable_compressed_data.len() as u32;

            let dstlen = 0x100000;
            let mut dstlen2: u32 = 0;

            let mut dst = Vec::with_capacity(dstlen as usize);
            let pdst = dst.as_mut_ptr();

            let _ntstatus = RtlDecompressBuffer(compression_format_lznt1 | compression_engine_standard, pdst, dstlen, psrc, srclen, &mut dstlen2);
        };
    });
}

#[bench]
fn bench_lznt1_decompress(b: &mut Bencher) {
    let compressed_data = include_bytes!("../tests/block1.compressed.bin");

    b.iter(|| {
        let _uncompressed = lzxpress::lznt1::decompress(compressed_data).unwrap();
    });
}