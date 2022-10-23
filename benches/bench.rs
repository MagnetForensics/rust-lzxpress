#![feature(test)]

extern crate lzxpress;
extern crate test;

use test::Bencher;

extern "C" {
    fn decompress_lznt1(
        in_buf: *const u8,
        in_buf_max_size: i32,
        out_buf: *mut u8,
        out_buf_max_size: i32,
        pout_buf_size: *mut i32,
    ) -> bool;
}

#[bench]
#[cfg(windows)]
fn bench_lznt1_decompress_rtl(b: &mut Bencher) {
    let compression_format_lznt1 = 0x0002 as u16;
    let compression_engine_standard = 0x0000 as u16;
    use ntapi::ntrtl::RtlDecompressBuffer;

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

            let _ntstatus = RtlDecompressBuffer(
                compression_format_lznt1 | compression_engine_standard,
                pdst,
                dstlen,
                psrc,
                srclen,
                &mut dstlen2,
            );

            dst.set_len(dstlen2 as usize);
        };
    });
}

#[bench]
fn bench_lznt1_decompress_cpp(b: &mut Bencher) {
    let compressed_data = include_bytes!("../tests/block1.compressed.bin");

    b.iter(|| {
        unsafe {
            let psrc = compressed_data.as_ptr();
            let srclen = compressed_data.len() as i32;

            let dstlen = 0x0010_0000;
            let mut dstlen2: i32 = 0;

            let mut dst = Vec::with_capacity(dstlen as usize);
            let pdst = dst.as_mut_ptr();

            let _status = decompress_lznt1(psrc, srclen, pdst, dstlen, &mut dstlen2);
            dst.set_len(dstlen2 as usize);
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

#[bench]
fn bench_lznt1_decompress2_push(b: &mut Bencher) {
    let compressed_data = include_bytes!("../tests/block1.compressed.bin");

    b.iter(|| {
        let dstlen = 0x0010_0000;
        let mut out_buf: Vec<u8> = Vec::with_capacity(dstlen);
        lzxpress::lznt1::decompress2_push(compressed_data, &mut out_buf).unwrap();
    });
}

#[bench]
fn bench_lznt1_decompress2_no_push(b: &mut Bencher) {
    let compressed_data = include_bytes!("../tests/block1.compressed.bin");

    b.iter(|| {
        let dstlen = 0x0010_0000;
        let mut out_buf: Vec<u8> = Vec::with_capacity(dstlen);
        unsafe {
            out_buf.set_len(dstlen);
        }
        lzxpress::lznt1::decompress2_no_push(compressed_data, &mut out_buf).unwrap();
    });
}

#[bench]
#[cfg(windows)]
fn bench_lzxpress_decompress_rtl(b: &mut Bencher) {
    let compression_format_xpress = 0x0003 as u16;
    let compression_engine_standard = 0x0000 as u16;
    use ntapi::ntrtl::RtlDecompressBuffer;

    let compressed_data = include_bytes!(
        "../tests/clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5724999789051904"
    );
    let mut mutable_compressed_data = compressed_data.to_vec();

    b.iter(|| {
        let _ret = unsafe {
            let psrc = mutable_compressed_data.as_mut_ptr();
            let srclen = mutable_compressed_data.len() as u32;

            let dstlen = 0x100000;
            let mut dstlen2: u32 = 0;

            let mut dst = Vec::with_capacity(dstlen as usize);
            let pdst = dst.as_mut_ptr();

            let _ntstatus = RtlDecompressBuffer(
                compression_format_xpress | compression_engine_standard,
                pdst,
                dstlen,
                psrc,
                srclen,
                &mut dstlen2,
            );
        };
    });
}

#[bench]
fn bench_lzxpress_decompress(b: &mut Bencher) {
    let compressed_data = include_bytes!(
        "../tests/clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5724999789051904"
    );

    b.iter(|| {
        let _uncompressed = lzxpress::data::decompress(compressed_data);
    });
}
