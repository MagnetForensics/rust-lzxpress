extern crate lzxpress;

use std::str;
// use lzxpress::error::LzxpressError;

const TEST_STRING: &'static str = "this is a test. and this is a test too\0\0\0\0";
const TEST_DATA: &'static [u8] = &[ 0x00, 0x20, 0x00, 0x04, 0x74, 0x68, 0x69, 0x73,
                0x20, 0x10, 0x00, 0x61, 0x20, 0x74, 0x65, 0x73,
                0x74, 0x2E, 0x20, 0x61, 0x6E, 0x64, 0x20, 0x9F,
                0x00, 0x04, 0x20, 0x74, 0x6F, 0x6F, 0x00, 0x00,
                0x00, 0x00 ];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress1() {
        let uncompressed = lzxpress::decompress(TEST_DATA).unwrap();

        if let Ok(s) = str::from_utf8(&uncompressed) {
            println!("{}", s);
        }

        let left = uncompressed.len();
        let right = TEST_STRING.len();
        assert!(left == right, "left = {}, right = {}", left, right);
        assert_eq!(uncompressed, TEST_STRING.as_bytes());
    }
}