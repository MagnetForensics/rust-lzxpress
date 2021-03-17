extern crate lzxpress;

use std::str;
use lzxpress::error::Error;

const TEST_STRING: &'static str = "this is a test. and this is a test too";
const TEST_DATA: &'static [u8] = &[ 
                0x00, 0x20, 0x00, 0x04, 0x74, 0x68, 0x69, 0x73,
                0x20, 0x10, 0x00, 0x61, 0x20, 0x74, 0x65, 0x73,
                0x74, 0x2E, 0x20, 0x61, 0x6E, 0x64, 0x20, 0x9F,
                0x00, 0x04, 0x20, 0x74, 0x6F, 0x6F ];

const TEST_OSSFUZZ_20083_DATA: &'static [u8] = include_bytes!("clusterfuzz-testcase-minimized-fuzz_ndr_drsuapi_TYPE_OUT-5724999789051904");

const TEST_STRING2: &'static str = "abcdefghijklmnopqrstuvwxyz";
const TEST_DATA2: &'static [u8] = &[ 
                0x00, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x64,
                0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c,
                0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74,
                0x75, 0x76, 0x77, 0x78, 0x79, 0x7a ];

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
        let result = lzxpress::data::decompress(TEST_OSSFUZZ_20083_DATA);

        match result {
            Err(Error::CorruptedData) => assert!(true),
            _ => panic!("This test should fail because of failed data.")
        }
    }

    #[test]
    fn test_compress1() {
        let compressed = lzxpress::data::compress(TEST_STRING.as_bytes()).unwrap();

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
}