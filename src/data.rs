use std::mem;
use std::cmp;

pub use crate::error::Error;

pub fn decompress(
    in_buf: &[u8]
) -> Result<Vec<u8>, Error>
{
    let mut out_idx:    usize = 0;
    let mut in_idx:     usize = 0;
    let mut nibble_idx: usize = 0;

    let mut indicator:     u32 = 0;
    let mut indicator_bit: u32 = 0;

    let mut length: usize;
    let mut offset: usize;

    let mut out_buf: Vec<u8> = Vec::new();

    while in_idx < in_buf.len() {
        if indicator_bit == 0 {
            indicator = u32::from(in_buf[in_idx + 3]) << 24
                        | u32::from(in_buf[in_idx + 2]) << 16
                        | u32::from(in_buf[in_idx + 1]) << 8
                        | u32::from(in_buf[in_idx]);

            in_idx = in_idx + mem::size_of::<u32>();

            indicator_bit = 32;
        }

        indicator_bit = indicator_bit - 1;

        // Check whether the bit specified by indicator_bit is set or not
        // set in indicator. For example, if indicator_bit has value 4
        // check whether the 4th bit of the value in indicator is set.
        if ((indicator >> indicator_bit) & 1) == 0 {
            if in_idx >= in_buf.len() {
                return Err(Error::MemLimit);
            }

            out_buf.push(in_buf[in_idx]);

            in_idx += mem::size_of::<u8>();
            out_idx += mem::size_of::<u8>();
        } else {
            if (in_idx + 1) >= in_buf.len() {
                return Err(Error::MemLimit);
            }

            length = usize::from(in_buf[in_idx + 1]) << 8
                    | usize::from(in_buf[in_idx]);

            in_idx = in_idx + mem::size_of::<u16>();

            offset = length / 8;
            length = length % 8;

            if length == 7 {
                if nibble_idx == 0 {
                    nibble_idx = in_idx;

                    if in_idx >= in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = (in_buf[in_idx] % 16).into();

                    in_idx = in_idx + mem::size_of::<u8>();
                } else {
                    if nibble_idx >= in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = (in_buf[nibble_idx] / 16).into();
                    nibble_idx = 0;
                }

                if length == 15 {
                    if in_idx >= in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = in_buf[in_idx].into();

                    in_idx = in_idx + mem::size_of::<u8>();

                    if length == 255 {
                        if (in_idx + 1) >= in_buf.len() {
                            return Err(Error::MemLimit);
                        }

                        length = usize::from(in_buf[in_idx + 1]) << 8
                                | usize::from(in_buf[in_idx]);

                        in_idx = in_idx + mem::size_of::<u16>();

                        length = length - (15 + 7);
                    }
                    length = length + 15;
                }
                length = length + 7;
            }

            length = length + 3;

            while length != 0 {
                if (offset + 1) > out_idx {
                    return Err(Error::CorruptedData);
                }

                out_buf.push(out_buf[out_idx - offset - 1]);

                out_idx = out_idx + mem::size_of::<u8>();
                length = length - mem::size_of::<u8>();
            }
        }
    }

    Ok(out_buf)
}

pub fn compress(
    uncompressed: &[u8]
) -> Result<Vec<u8>, Error>
{

    let mut uncompressed_pos: usize = 0;
    let mut compressed_pos:   usize;
    let mut byte_left:        usize;

    let mut max_offset:  usize;
    let mut best_offset: usize;

    let mut max_len:  usize;
    let mut best_len: usize;

    let mut indic:        u32 = 0;
    let mut indic_pos:    usize;
    let mut indic_bit:    u32 = 0;
    let mut nibble_index: usize = 0;
    
    let mut metadata_size: usize;
    let mut metadata:      usize;
    let mut dest_offset:   usize;

    let mut str1_offset: usize;
    let mut str2_offset: usize;

    let mut compressed: Vec<u8> = Vec::new();

    // Metadata placeholder
    compressed.push(0);
    compressed.push(0);
    compressed.push(0);
    compressed.push(0);

    compressed_pos = mem::size_of::<u32>();
    indic_pos = 0;

    byte_left = uncompressed.len();

    while byte_left >= 3 {
        let mut found: bool = false;

        max_offset = uncompressed_pos;

        str1_offset = uncompressed_pos;

        best_len = 2;
        best_offset = 0;

        max_offset = cmp::min(0x1FFF, max_offset);

        // search for the longest match in the window for the lookahead buffer
        for offset in 1..=max_offset {
            let len: usize = 0;
            str2_offset = str1_offset - offset;

            // maximum len we can encode into metadata
            max_len = cmp::min(255 + 15 + 7 + 3, byte_left);

            for len in 0..max_len {
                if uncompressed[str1_offset + len] != uncompressed[str2_offset + len] {
                    break;
                }
            }

            // We check if len is better than the value found before, including the
            // sequence of identical bytes
            if len > best_len {
                found = true;
                best_len = len;
                best_offset = offset;
            }
        }

        if found {
            metadata_size = 0;
            dest_offset = compressed_pos;

            if best_len < 10 {
                // Classical meta-data
                metadata = ((best_offset - 1) << 3) | (best_len - 3);
                compressed[dest_offset + metadata_size] = metadata as u8;
                compressed[dest_offset + metadata_size + 1] = (metadata >> 8) as u8;
                metadata_size = metadata_size + mem::size_of::<u16>();
            } else {
                metadata = ((best_offset - 1) << 3) | 7;
                compressed[dest_offset + metadata_size] = metadata as u8;
                compressed[dest_offset + metadata_size + 1] = (metadata >> 8) as u8;
                metadata_size = metadata_size + mem::size_of::<u16>();

                if best_len < (15 + 7 + 3) {
                    // Shared byte
                    if nibble_index == 0 {
                        compressed[compressed_pos + metadata_size] = ((best_len - (3 + 7)) & 0xF) as u8;
                        metadata_size = metadata_size + mem::size_of::<u8>();
                    } else {
                        compressed[nibble_index] = compressed[nibble_index] & 0xF;
                        compressed[nibble_index] = compressed[nibble_index] | ((best_len - (3 + 7)) * 16) as u8;
                    }
                } else if best_len < (3 + 7 + 15 + 255) {
                    // Shared byte
                    if nibble_index == 0 {
                        compressed[compressed_pos + metadata_size] = 15;
                        metadata_size = metadata_size + mem::size_of::<u8>();
                    } else {
                        compressed[nibble_index] = compressed[nibble_index] & 0xF;
                        compressed[nibble_index] = compressed[nibble_index] | (15 * 16);
                    }

                    // Additional best_len
                    compressed[compressed_pos + metadata_size] = (best_len - (3 + 7 + 15)) as u8;
                    metadata_size = metadata_size + mem::size_of::<u8>();
                } else {
                    // Shared byte
                    if nibble_index == 0 {
                        compressed[compressed_pos + metadata_size] = compressed[compressed_pos + metadata_size] | 15;
                        metadata_size = metadata_size + mem::size_of::<u8>();
                    } else {
                        compressed[nibble_index] = compressed[nibble_index] | (15 << 4);
                    }

                    // Additional best_len
                    compressed[compressed_pos + metadata_size] = 255;
                    metadata_size = metadata_size + mem::size_of::<u8>();

                    compressed[compressed_pos + metadata_size] = (best_len - 3) as u8;
                    compressed[compressed_pos + metadata_size + 1] = ((best_len - 3) >> 8) as u8;
                    metadata_size = metadata_size + mem::size_of::<u16>();
                }
            }

            indic = indic | (1 << (32 - ((indic_bit % 32) + 1)));

            if best_len > 9 {
                if nibble_index == 0 {
                    nibble_index = compressed_pos + mem::size_of::<u16>();
                } else {
                    nibble_index = 0;
                }
            }

            compressed_pos = compressed_pos + metadata_size;
            uncompressed_pos = uncompressed_pos + best_len;
            byte_left = byte_left - best_len;
        } else {
            compressed.push(uncompressed[uncompressed_pos]);
            compressed_pos = compressed_pos + 1;
            uncompressed_pos = uncompressed_pos + 1;

            byte_left = byte_left - 1;
        }

        indic_bit = indic_bit + 1;

        if ((indic_bit - 1) % 32) > (indic_bit % 32) {
            compressed[indic_pos + 0] = indic as u8;
            compressed[indic_pos + 1] = (indic >> 8) as u8;
            compressed[indic_pos + 2] = (indic >> 16) as u8;
            compressed[indic_pos + 3] = (indic >> 24) as u8;

            indic = 0;
            indic_pos = compressed_pos;
            // Metadata placeholder
            compressed.push(0);
            compressed.push(0);
            compressed.push(0);
            compressed.push(0);
            compressed_pos = compressed_pos + mem::size_of::<u32>();
        }
    }

    while uncompressed_pos < uncompressed.len() {
        compressed.push(uncompressed[uncompressed_pos]);
        indic_bit = indic_bit + 1;

        uncompressed_pos = uncompressed_pos + 1;
        compressed_pos = compressed_pos + 1;
        if ((indic_bit - 1) % 32) > (indic_bit % 32) {
            compressed[indic_pos + 0] = indic as u8;
            compressed[indic_pos + 1] = (indic >> 8) as u8;
            compressed[indic_pos + 2] = (indic >> 16) as u8;
            compressed[indic_pos + 3] = (indic >> 24) as u8;

            indic = 0;
            indic_pos = compressed_pos;
            compressed_pos = compressed_pos + mem::size_of::<u32>();
        }
    }

    if (indic_bit % 32) > 0 {
        while (indic_bit % 32) != 0 {
            indic |= 0 << (32 - ((indic_bit % 32) + 1));
            indic_bit = indic_bit + 1;
        }

        compressed[indic_pos + 0] = indic as u8;
        compressed[indic_pos + 1] = (indic >> 8) as u8;
        compressed[indic_pos + 2] = (indic >> 16) as u8;
        compressed[indic_pos + 3] = (indic >> 24) as u8;

        // compressed_pos = compressed_pos + mem::size_of::<u32>();
    }

    Ok(compressed)
}