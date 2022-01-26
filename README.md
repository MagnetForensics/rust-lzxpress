# rust-lzxpress
![status](https://github.com/comaeio/rust-lzxpress/actions/workflows/rust.yml/badge.svg)

## [MS-XCA]: Xpress Compression Algorithm
### Introduction
The [Xpress Compression Algorithm](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/a8b7cb0a-92a6-4187-a23b-5e14273b96f8) has three variants, all designed for speed.
The fastest variant, Plain LZ77, implements the LZ77 algorithm ([UASDC](https://go.microsoft.com/fwlink/?LinkId=90549)).
A slower variant, LZ77+Huffman, adds a Huffman encoding pass on the LZ77 data.
A third variant, LZNT1, implements LZ77 without the Huffman encoding pass of the second variant,
but with an encoding process less complex than Plain LZ77.
### Overview
This algorithm efficiently compresses data that contain repeated byte sequences. It is not designed to compress image, audio, or video data. Between the trade-offs of compressed size and CPU cost, it heavily emphasizes low CPU cost.
### Relationship to Protocols and Other Algorithms
This algorithm does not depend on any other algorithms or protocols. It is a compression method designed to have minimal CPU overhead for compression and decompression. A protocol that depends on this algorithm would typically need to transfer significant amounts of data that cannot be easily precompressed by another algorithm having a better compression ratio.
### Applicability Statement
This algorithm is appropriate for any protocol that transfers large amounts of easily compressible textlike data, such as HTML, source code, or log files. Protocols use this algorithm to reduce the number of bits transferred.

## This library
This crate provides a simple interface to Microsoft Xpress compression algorithm.  Microsoft Xpress Compression Algorithm is more commonly known
as LZXpress. This algorithm efficiently compresses data that contain repeated byte sequences. It is not designed to
compress image, audio, or video data. Between the trade-offs of compressed size and CPU cost, it
heavily emphasizes low CPU cost. It is mainly used by Microsoft features or protocols such as Microsoft Windows hibernation file, [Microsoft SMB protocol](https://ftp.samba.org/pub/unpacked/samba_master/lib/compression/lzxpress.c)
or even [Microsoft Windows 10 compressed memory management](https://www.fireeye.com/content/dam/fireeye-www/blog/pdfs/finding-evil-in-windows-10-compressed-memory-wp.pdf).

`decompress`/`compress` are an easy to use functions for simple use cases.

By default, LZXpress on Windows uses the Plain LZ77 Algorithm. You can read more about it in the [MS-XCA] documentation under the `2.4	Plain LZ77 Decompression Algorithm Details` and `2.3	Plain LZ77 Compression Algorithm Details` sections.

### Example ###
Cargo.toml:
```toml
[dependencies]
rust-lzxpress = "0.7.1"
```
main.rs:
```Rust
extern crate lzxpress;

use lzxpress;

const TEST_STRING: &'static str = "abcdefghijklmnopqrstuvwxyz";
const TEST_DATA: &'static [u8] = &[ 
                0x3f, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x64,
                0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c,
                0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74,
                0x75, 0x76, 0x77, 0x78, 0x79, 0x7a ];

const TEST_LZNT1_COMPRESSED_DATA: &'static [u8] = include_bytes!("block1.compressed.bin");

fn main() {
    let uncompressed = lzxpress::data::decompress(TEST_DATA).unwrap();

    if let Ok(s) = str::from_utf8(&uncompressed) {
        println!("{}", s);
    }

    let compressed = lzxpress::data::compress(TEST_STRING.as_bytes()).unwrap();
    let uncompressed2 = lzxpress::data::decompress(compressed.as_slice()).unwrap();
    if let Ok(s) = str::from_utf8(&uncompressed2) {
        println!("{}", s);
    }

    // LZNT1
    let uncompressed_lznt1 = lzxpress::lznt1::decompress(TEST_LZNT1_COMPRESSED_DATA).unwrap();
}
```

### Benchmarks
This rust implementation of LZNT1 decompression is currently twice slower than the native ntdll rtl RtlDecompressBuffer() implementation. 
And still is 50% slower thant the equivalent implemented in C under `tests/lznt1.c!decompress_lznt1()`.

```
running 7 tests
test bench_lznt1_decompress          ... bench:   5,985,430 ns/iter (+/- 348,885)
test bench_lznt1_decompress2_no_push ... bench:   4,526,545 ns/iter (+/- 32,742)
test bench_lznt1_decompress2_push    ... bench:   5,904,550 ns/iter (+/- 444,512)
test bench_lznt1_decompress_cpp      ... bench:   3,947,145 ns/iter (+/- 111,184)
test bench_lznt1_decompress_rtl      ... bench:   2,838,820 ns/iter (+/- 175,803)
test bench_lzxpress_decompress       ... bench:         320 ns/iter (+/- 14)
test bench_lzxpress_decompress_rtl   ... bench:      16,299 ns/iter (+/- 3,853)
```