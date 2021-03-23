extern crate lzxpress;

use std::str;

const TEST_STRING: &'static str = "this is a test. and this is a test too";
const TEST_DATA: &'static [u8] = &[ 
                0x00, 0x20, 0x00, 0x04, 0x74, 0x68, 0x69, 0x73,
                0x20, 0x10, 0x00, 0x61, 0x20, 0x74, 0x65, 0x73,
                0x74, 0x2E, 0x20, 0x61, 0x6E, 0x64, 0x20, 0x9F,
                0x00, 0x04, 0x20, 0x74, 0x6F, 0x6F ];

// Issue 19382: samba:fuzz_lzxpress: Heap-buffer-overflow in lzxpress_decompress
// https://bugs.chromium.org/p/oss-fuzz/issues/detail?id=20083&q=samba&can=2
const TEST_OSSFUZZ_20083_DATA: &'static [u8] = include_bytes!("clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5724999789051904");
const TEST_OSSFUZZ_5698056963227648_DATA: &'static [u8] = include_bytes!("clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5698056963227648");

// Examples: https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/72da4f8d-2ba3-437d-b772-2e4173713a0b?redirectedfrom=MSDN
const TEST_STRING2: &'static str = "abcdefghijklmnopqrstuvwxyz";
const TEST_DATA2: &'static [u8] = &[ 
                0x3f, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x64,
                0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c,
                0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74,
                0x75, 0x76, 0x77, 0x78, 0x79, 0x7a ];

const TEST_STRING3: &'static str = "abcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabc";
const TEST_DATA3: &'static [u8] = &[ 
                0xff, 0xff, 0xff, 0x1f, 0x61, 0x62, 0x63, 0x17,
                0x00, 0x0f, 0xff, 0x26, 0x01 ];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress1() {
        let uncompressed = lzxpress::data::decompress(TEST_DATA).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }

        assert!(uncompressed.len() == TEST_STRING.len(), "uncompressed.len = {}, TEST_STRING.len = {}", uncompressed.len(), TEST_STRING.len());
        assert_eq!(uncompressed, TEST_STRING.as_bytes());
    }


    #[test]
    fn test_decompress2() {
        let uncompressed = lzxpress::data::decompress(TEST_DATA2).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }

        assert!(uncompressed.len() == TEST_STRING2.len(), "uncompressed.len = {}, TEST_STRING.len = {}", uncompressed.len(), TEST_STRING2.len());
        assert_eq!(uncompressed, TEST_STRING2.as_bytes());
    }

    #[test]
    fn test_decompress3() {
        let uncompressed = lzxpress::data::decompress(TEST_DATA3).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }

        assert!(uncompressed.len() == TEST_STRING3.len(), "uncompressed.len = {}, TEST_STRING.len = {}", uncompressed.len(), TEST_STRING3.len());
        assert_eq!(uncompressed, TEST_STRING3.as_bytes());
    }

    #[test]
    fn test_decompress_err1() {
        let result = lzxpress::data::decompress(TEST_OSSFUZZ_20083_DATA);

        match result {
            Err(_e) => assert!(true),
            _ => panic!("This test should fail because of failed data.")
        }
    }

    #[test]
    fn test_decompress_err2() {
        let result = lzxpress::data::decompress(TEST_OSSFUZZ_5698056963227648_DATA);

        match result {
            Err(_e) => assert!(true),
            _ => panic!("This test should fail because of failed data.")
        }
    }

    #[test]
    fn test_compress1() {
        let compressed = lzxpress::data::compress(TEST_STRING.as_bytes()).unwrap();
        // this come from smb legacy test and implementation, so the output will be different.
        // assert_eq!(compressed, TEST_DATA);

        let uncompressed = lzxpress::data::decompress(compressed.as_slice()).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }

        assert_eq!(uncompressed, TEST_STRING.as_bytes());
        assert!(uncompressed.len() == TEST_STRING.len(), "uncompressed.len = {}, TEST_STRING.len = {}", uncompressed.len(), TEST_STRING.len());

    }

    #[test]
    fn test_compress2() {
        let compressed = lzxpress::data::compress(TEST_STRING2.as_bytes()).unwrap();

        assert!(compressed.len() == TEST_DATA2.len(), "compressed.len = {}, TEST_DATA2.len = {}", compressed.len(), TEST_DATA2.len());
        assert_eq!(compressed, TEST_DATA2);

        let uncompressed = lzxpress::data::decompress(compressed.as_slice()).unwrap();
        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }
        assert_eq!(uncompressed, TEST_STRING2.as_bytes());
    }

    #[test]
    fn test_compress3() {
        let compressed = lzxpress::data::compress(TEST_STRING3.as_bytes()).unwrap();

        println!("{}", TEST_STRING3);

        if let Ok(s) = str::from_utf8(&compressed) {
            println!("{}", s);
        }

        assert!(compressed.len() == TEST_DATA3.len(), "compressed.len = {}, TEST_DATA2.len = {}", compressed.len(), TEST_DATA3.len());
        assert_eq!(compressed, TEST_DATA3);

        let uncompressed = lzxpress::data::decompress(compressed.as_slice()).unwrap();
        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }
        assert_eq!(uncompressed, TEST_STRING3.as_bytes());
    }
}