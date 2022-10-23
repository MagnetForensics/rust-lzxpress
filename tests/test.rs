extern crate lzxpress;

use std::str;

const TEST_STRING: &str = "this is a test. and this is a test too";
const TEST_DATA: &[u8] = &[
    0x00, 0x20, 0x00, 0x04, 0x74, 0x68, 0x69, 0x73, 0x20, 0x10, 0x00, 0x61, 0x20, 0x74, 0x65, 0x73,
    0x74, 0x2E, 0x20, 0x61, 0x6E, 0x64, 0x20, 0x9F, 0x00, 0x04, 0x20, 0x74, 0x6F, 0x6F,
];

// Issue 19382: samba:fuzz_lzxpress: Heap-buffer-overflow in lzxpress_decompress
// https://bugs.chromium.org/p/oss-fuzz/issues/detail?id=20083&q=samba&can=2
const TEST_OSSFUZZ_20083_DATA: &[u8] =
    include_bytes!("clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5724999789051904");
const TEST_OSSFUZZ_5698056963227648_DATA: &[u8] =
    include_bytes!("clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5698056963227648");

const TEST_FAST_FFFFFFFF_COMPRESSED_DATA: &[u8] = include_bytes!("fast.compressed.ffffffff.bin");
const TEST_FAST_FFFFFFFF_UNCOMPRESSED_DATA: &[u8] =
    include_bytes!("fast.uncompressed.ffffffff.bin");

// Examples: https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/72da4f8d-2ba3-437d-b772-2e4173713a0b?redirectedfrom=MSDN
const TEST_STRING2: &str = "abcdefghijklmnopqrstuvwxyz";
const TEST_DATA2: &[u8] = &[
    0x3f, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c,
    0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a,
];

const TEST_STRING3: &str = "abcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabc";
const TEST_DATA3: &[u8] = &[
    0xff, 0xff, 0xff, 0x1f, 0x61, 0x62, 0x63, 0x17, 0x00, 0x0f, 0xff, 0x26, 0x01,
];

const TEST_LZNT1_STRING1: &str = "F# F# G A A G F# E D D E F# F# E E F# F# G A A G F# E D D E F# E D D E E F# D E F# G F# D E F# G F# E D E A F# F# G A A G F# E D D E F# E D D\0";
const TEST_LZNT1_DATA1: &[u8] = &[
    0x38, 0xb0, 0x88, 0x46, 0x23, 0x20, 0x00, 0x20, 0x47, 0x20, 0x41, 0x00, 0x10, 0xa2, 0x47, 0x01,
    0xa0, 0x45, 0x20, 0x44, 0x00, 0x08, 0x45, 0x01, 0x50, 0x79, 0x00, 0xc0, 0x45, 0x20, 0x05, 0x24,
    0x13, 0x88, 0x05, 0xb4, 0x02, 0x4a, 0x44, 0xef, 0x03, 0x58, 0x02, 0x8c, 0x09, 0x16, 0x01, 0x48,
    0x45, 0x00, 0xbe, 0x00, 0x9e, 0x00, 0x04, 0x01, 0x18, 0x90, 0x00,
];

const TEST_LZNT1_COMPRESSED_DATA: &[u8] = include_bytes!("block1.compressed.bin");
const TEST_LZNT1_UNCOMPRESSED_DATA: &[u8] = include_bytes!("block1.uncompressed.bin");

extern "C" {
    fn decompress_lznt1(
        in_buf: *const u8,
        in_buf_max_size: i32,
        out_buf: *mut u8,
        out_buf_max_size: i32,
        pout_buf_size: *mut i32,
    ) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress1() {
        let uncompressed = lzxpress::data::decompress(TEST_DATA).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }

        assert!(
            uncompressed.len() == TEST_STRING.len(),
            "uncompressed.len = {}, TEST_STRING.len = {}",
            uncompressed.len(),
            TEST_STRING.len()
        );
        assert_eq!(uncompressed, TEST_STRING.as_bytes());
    }

    #[test]
    fn test_decompress2() {
        let uncompressed = lzxpress::data::decompress(TEST_DATA2).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }

        assert!(
            uncompressed.len() == TEST_STRING2.len(),
            "uncompressed.len = {}, TEST_STRING.len = {}",
            uncompressed.len(),
            TEST_STRING2.len()
        );
        assert_eq!(uncompressed, TEST_STRING2.as_bytes());
    }

    #[test]
    fn test_decompress3() {
        let uncompressed = lzxpress::data::decompress(TEST_DATA3).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }

        assert!(
            uncompressed.len() == TEST_STRING3.len(),
            "uncompressed.len = {}, TEST_STRING.len = {}",
            uncompressed.len(),
            TEST_STRING3.len()
        );
        assert_eq!(uncompressed, TEST_STRING3.as_bytes());
    }

    #[test]
    fn test_decompress4() {
        assert_eq!(
            &lzxpress::data::decompress(TEST_FAST_FFFFFFFF_COMPRESSED_DATA).unwrap(),
            TEST_FAST_FFFFFFFF_UNCOMPRESSED_DATA
        );
    }

    #[test]
    fn test_decompress_err1() {
        let result = lzxpress::data::decompress(TEST_OSSFUZZ_20083_DATA);

        match result {
            Err(_e) => assert!(true),
            _ => panic!("This test should fail because of failed data."),
        }
    }

    #[test]
    fn test_decompress_err2() {
        let result = lzxpress::data::decompress(TEST_OSSFUZZ_5698056963227648_DATA);

        match result {
            Err(_e) => assert!(true),
            _ => panic!("This test should fail because of failed data."),
        }
    }

    #[test]
    fn test_compress1() {
        let compressed = lzxpress::data::compress(TEST_STRING.as_bytes()).unwrap();
        // this come from smb legacy test and implementation, so the output will be different.
        // assert_eq!(compressed, TEST_DATA);

        let uncompressed = lzxpress::data::decompress(compressed.as_slice()).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }

        assert_eq!(uncompressed, TEST_STRING.as_bytes());
        assert!(
            uncompressed.len() == TEST_STRING.len(),
            "uncompressed.len = {}, TEST_STRING.len = {}",
            uncompressed.len(),
            TEST_STRING.len()
        );
    }

    #[test]
    fn test_compress2() {
        let compressed = lzxpress::data::compress(TEST_STRING2.as_bytes()).unwrap();

        assert!(
            compressed.len() == TEST_DATA2.len(),
            "compressed.len = {}, TEST_DATA2.len = {}",
            compressed.len(),
            TEST_DATA2.len()
        );
        assert_eq!(compressed, TEST_DATA2);

        let uncompressed = lzxpress::data::decompress(compressed.as_slice()).unwrap();
        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }
        assert_eq!(uncompressed, TEST_STRING2.as_bytes());
    }

    #[test]
    fn test_compress3() {
        let compressed = lzxpress::data::compress(TEST_STRING3.as_bytes()).unwrap();

        println!("{TEST_STRING3}");

        if let Ok(s) = str::from_utf8(&compressed) {
            println!("{s}");
        }

        assert!(
            compressed.len() == TEST_DATA3.len(),
            "compressed.len = {}, TEST_DATA2.len = {}",
            compressed.len(),
            TEST_DATA3.len()
        );
        assert_eq!(compressed, TEST_DATA3);

        let uncompressed = lzxpress::data::decompress(compressed.as_slice()).unwrap();
        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }
        assert_eq!(uncompressed, TEST_STRING3.as_bytes());
    }

    #[test]
    fn test_lznt1_decompress1() {
        let uncompressed = lzxpress::lznt1::decompress(TEST_LZNT1_DATA1).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{s}");
        }

        assert!(
            uncompressed.len() == TEST_LZNT1_STRING1.len(),
            "uncompressed.len = {}, TEST_LZNT1_STRING1.len = {}",
            uncompressed.len(),
            TEST_LZNT1_STRING1.len()
        );
        assert_eq!(uncompressed, TEST_LZNT1_STRING1.as_bytes());
    }

    #[test]
    fn test_lznt1_decompress2() {
        let uncompressed = lzxpress::lznt1::decompress(TEST_LZNT1_COMPRESSED_DATA).unwrap();
        // let c: &[u8] = &uncompressed;
        // let mut output = File::create("rust.uncompressed.bin").expect("Unable to open");
        // output.write(c).expect("Unable to open");

        assert!(
            uncompressed.len() == 0x0010_0000,
            "uncompressed.len = {} (expected len = 0x10000)",
            uncompressed.len()
        );
        assert_eq!(uncompressed, TEST_LZNT1_UNCOMPRESSED_DATA);
    }

    #[test]
    #[cfg(windows)]
    fn test_lznt1_decompress_vs_rtl() {
        unsafe {
            let uncompressed_local =
                lzxpress::lznt1::decompress(TEST_LZNT1_COMPRESSED_DATA).unwrap();

            let compression_format_lznt1 = 0x0002 as u16;
            let compression_engine_standard = 0x0000 as u16;
            use ntapi::ntrtl::RtlDecompressBuffer;

            let mut mutable_compressed_data = TEST_LZNT1_COMPRESSED_DATA.to_vec();
            let psrc = mutable_compressed_data.as_mut_ptr();
            let srclen = mutable_compressed_data.len() as u32;

            let dstlen = 0x100000;
            let mut dstlen2: u32 = 0;

            let mut uncompressed_rtl = Vec::with_capacity(dstlen as usize);
            let pdst = uncompressed_rtl.as_mut_ptr();

            let _ntstatus = RtlDecompressBuffer(
                compression_format_lznt1 | compression_engine_standard,
                pdst,
                dstlen,
                psrc,
                srclen,
                &mut dstlen2,
            );
            uncompressed_rtl.set_len(dstlen2 as usize);

            assert!(
                dstlen == dstlen2,
                "dstlen = {} and dstlen2 = {})",
                dstlen,
                dstlen2
            );
            assert!(
                uncompressed_local.len() == uncompressed_rtl.len(),
                "uncompressed_local.len = {} and uncompressed_rtl = {})",
                uncompressed_local.len(),
                uncompressed_rtl.len()
            );
            assert_eq!(uncompressed_local, uncompressed_rtl);
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_lznt1_decompress_vs_no_push_optz() {
        unsafe {
            let uncompressed_local =
                lzxpress::lznt1::decompress(TEST_LZNT1_COMPRESSED_DATA).unwrap();

            let dstlen = 0x100000;
            let mut out_buf: Vec<u8> = Vec::with_capacity(dstlen);
            out_buf.set_len(dstlen);

            let _res =
                lzxpress::lznt1::decompress2_no_push(TEST_LZNT1_COMPRESSED_DATA, &mut out_buf);

            assert!(
                uncompressed_local.len() == out_buf.len(),
                "uncompressed_local.len = {} and out_buf = {})",
                uncompressed_local.len(),
                out_buf.len()
            );
            assert_eq!(uncompressed_local, out_buf);
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_lznt1_decompress_vs_cpp() {
        unsafe {
            let uncompressed_local =
                lzxpress::lznt1::decompress(TEST_LZNT1_COMPRESSED_DATA).unwrap();

            let psrc = TEST_LZNT1_COMPRESSED_DATA.as_ptr();
            let srclen = TEST_LZNT1_COMPRESSED_DATA.len() as i32;

            let dstlen = 0x100000;
            let mut dstlen2: i32 = 0;

            let mut dst = Vec::with_capacity(dstlen as usize);
            let pdst = dst.as_mut_ptr();
            let _status = decompress_lznt1(psrc, srclen, pdst, dstlen, &mut dstlen2);
            dst.set_len(dstlen2 as usize);

            assert!(
                dstlen == dstlen2,
                "dstlen = {} and dstlen2 = {} and srclen = {}",
                dstlen,
                dstlen2,
                srclen
            );
            assert!(
                uncompressed_local.len() == dst.len(),
                "uncompressed_local.len = {} and uncompressed_rtl = {})",
                uncompressed_local.len(),
                dst.len()
            );
            assert_eq!(uncompressed_local, dst);
        }
    }
}
