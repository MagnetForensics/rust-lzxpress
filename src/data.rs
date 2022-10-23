use std::cmp;
use std::mem;

pub use crate::error::Error;

macro_rules! store32le {
    ($dst:expr,$idx:expr,$val:expr) => {{
        $dst[$idx + 0] = $val as u8;
        $dst[$idx + 1] = ($val >> 8) as u8;
        $dst[$idx + 2] = ($val >> 16) as u8;
        $dst[$idx + 3] = ($val >> 24) as u8;
    }};
}

macro_rules! load16le {
    ($dst:expr,$src:expr,$idx:expr) => {{
        $dst = (u32::from($src[$idx + 1]) << 8 | u32::from($src[$idx])) as usize;
    }};
}

macro_rules! load32le {
    ($dst:expr,$src:expr,$idx:expr) => {{
        $dst = ((u32::from($src[$idx + 3]) << 24)
            | (u32::from($src[$idx + 2]) << 16)
            | (u32::from($src[$idx + 1]) << 8)
            | u32::from($src[$idx])) as usize;
    }};
}

pub fn decompress(in_buf: &[u8]) -> Result<Vec<u8>, Error> {
    let mut out_idx: usize = 0;
    let mut in_idx: usize = 0;
    let mut nibble_idx: usize = 0;

    let mut flags: usize = 0;
    let mut flag_count: usize = 0;

    let mut length: usize;
    let mut offset: usize;

    let mut out_buf: Vec<u8> = Vec::new();

    while in_idx < in_buf.len() {
        if flag_count == 0 {
            if (in_idx + 3) >= in_buf.len() {
                return Err(Error::MemLimit);
            }

            load32le!(flags, in_buf, in_idx);
            in_idx += mem::size_of::<u32>();
            flag_count = 32;
        }

        flag_count -= 1;

        // Check whether the bit specified by flag_count is set or not
        // set in flags. For example, if flag_count has value 4
        // check whether the 4th bit of the value in flags is set.
        if (flags & (1 << flag_count)) == 0 {
            if in_idx >= in_buf.len() {
                return Err(Error::MemLimit);
            }
            out_buf.push(in_buf[in_idx]);

            in_idx += mem::size_of::<u8>();
            out_idx += mem::size_of::<u8>();
        } else {
            if in_idx == in_buf.len() {
                // [MS-XCA] - v20210625, 2.4.4
                break;
            }

            if (in_idx + 1) > in_buf.len() {
                return Err(Error::MemLimit);
            }

            load16le!(length, in_buf, in_idx);
            in_idx += mem::size_of::<u16>();

            offset = (length / 8) + 1;
            length %= 8;

            if length == 7 {
                if nibble_idx == 0 {
                    if in_idx > in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = (in_buf[in_idx] % 16).into();
                    nibble_idx = in_idx;
                    in_idx += mem::size_of::<u8>();
                } else {
                    if nibble_idx > in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = (in_buf[nibble_idx] / 16).into();
                    nibble_idx = 0;
                }

                if length == 15 {
                    if in_idx > in_buf.len() {
                        return Err(Error::MemLimit);
                    }

                    length = in_buf[in_idx].into();
                    in_idx += mem::size_of::<u8>();

                    if length == 255 {
                        if (in_idx + 1) > in_buf.len() {
                            return Err(Error::MemLimit);
                        }

                        load16le!(length, in_buf, in_idx);
                        in_idx += mem::size_of::<u16>();

                        if length == 0 {
                            load32le!(length, in_buf, in_idx);
                            in_idx += mem::size_of::<u32>();
                        }

                        if length < 15 + 7 {
                            return Err(Error::CorruptedData);
                        }
                        length -= 15 + 7;
                    }
                    length += 15;
                }
                length += 7;
            }
            length += 3;

            for _i in 0..length {
                if offset > out_idx {
                    return Err(Error::CorruptedData);
                }

                out_buf.push(out_buf[out_idx - offset]);
                out_idx += mem::size_of::<u8>();
            }
        }
    }

    Ok(out_buf)
}

pub fn compress(in_buf: &[u8]) -> Result<Vec<u8>, Error> {
    let mut in_idx: usize = 0;
    let mut out_idx: usize;
    let mut byte_left: usize;

    let mut max_off: usize;
    let mut match_off: usize;

    let mut max_len: usize;
    let mut best_len: usize;

    let mut flags: u32 = 0;
    let mut flag_count: u32 = 0;
    let mut flag_out_off: usize = 0;
    let mut nibble_index: usize = 0;

    let mut metadata_size: usize;
    let mut metadata: usize;
    let mut _dest_off: usize;

    let mut str1_off: usize;
    let mut str2_off: usize;

    let mut out_buf: Vec<u8> = Vec::new();

    // Flag placeholder
    out_buf.push(0);
    out_buf.push(0);
    out_buf.push(0);
    out_buf.push(0);
    out_idx = mem::size_of::<u32>();

    while in_idx < in_buf.len() {
        let mut found: bool = false;
        byte_left = in_buf.len() - in_idx;
        max_off = in_idx;

        str1_off = in_idx;

        best_len = 2;
        match_off = 0;

        max_off = cmp::min(8192, max_off);

        // search for the longest match in the window for the lookahead buffer
        for offset in 1..=max_off {
            let mut len = 0;
            str2_off = str1_off - offset;

            // maximum len we can encode into metadata
            max_len = cmp::min(8192, byte_left);

            for i in 0..max_len {
                if in_buf[str1_off + i] != in_buf[str2_off + i] {
                    break;
                }
                len = i + 1;
            }

            // We check if len is better than the value found before, including the
            // sequence of identical bytes
            if len > best_len {
                found = true;
                best_len = len;
                match_off = offset;
            }
        }

        if !found {
            out_buf.push(in_buf[in_idx]);
            out_idx += 1;
            in_idx += 1;

            flags <<= 1;
            flag_count += 1;
            if flag_count == 32 {
                store32le!(out_buf, flag_out_off, flags);
                flag_count = 0;
                flag_out_off = out_idx;
                out_buf.push(0);
                out_buf.push(0);
                out_buf.push(0);
                out_buf.push(0);
                out_idx += mem::size_of::<u32>();
            }
        } else {
            let mut match_len = best_len;
            metadata_size = 0;

            match_len -= 3;
            match_off -= 1;

            if match_len < 7 {
                // Classical meta-data
                metadata = (match_off << 3) + match_len;
                out_buf.push(metadata as u8);
                out_buf.push((metadata >> 8) as u8);
                metadata_size += mem::size_of::<u16>();
            } else {
                let mut has_extra_len: bool = false;

                metadata = (match_off << 3) | 7;
                out_buf.push(metadata as u8);
                out_buf.push((metadata >> 8) as u8);
                metadata_size += mem::size_of::<u16>();

                match_len -= 7;

                if nibble_index == 0 {
                    nibble_index = out_idx;
                    if match_len < 15 {
                        out_buf.push(match_len as u8);
                        metadata_size += mem::size_of::<u8>();
                    } else {
                        out_buf.push(15);
                        metadata_size += mem::size_of::<u8>();

                        has_extra_len = true;
                    }
                } else if match_len < 15 {
                    out_buf[nibble_index] |= (match_len << 4) as u8;
                    nibble_index = 0;
                } else {
                    out_buf[nibble_index] |= (15 << 4) as u8;
                    nibble_index = 0;

                    has_extra_len = true;
                }

                if has_extra_len {
                    match_len -= 15;

                    if match_len < 255 {
                        out_buf.push(match_len as u8);
                        metadata_size += mem::size_of::<u8>();
                    } else {
                        out_buf.push(255);
                        metadata_size += mem::size_of::<u8>();

                        match_len += 7 + 15;

                        if match_len < (1 << 16) {
                            out_buf.push(match_len as u8);
                            out_buf.push((match_len >> 8) as u8);
                            metadata_size += mem::size_of::<u16>();
                        } else {
                            out_buf.push(0);
                            out_buf.push(0);
                            metadata_size += mem::size_of::<u16>();
                            out_buf.push(match_len as u8);
                            out_buf.push((match_len >> 8) as u8);
                            out_buf.push((match_len >> 16) as u8);
                            out_buf.push((match_len >> 24) as u8);
                            metadata_size += mem::size_of::<u32>();
                        }
                    }
                }
            }

            flags = (flags << 1) | 1;
            flag_count += 1;
            if flag_count == 32 {
                store32le!(out_buf, flag_out_off, flags);
                flag_count = 0;
                flag_out_off = out_idx;
                out_buf.push(0);
                out_buf.push(0);
                out_buf.push(0);
                out_buf.push(0);
                out_idx += mem::size_of::<u32>();
            }

            out_idx += metadata_size;
            in_idx += best_len;
        }
    }

    flags <<= 32 - flag_count;
    flags |= (1 << (32 - flag_count)) - 1;
    store32le!(out_buf, flag_out_off, flags);

    Ok(out_buf)
}
