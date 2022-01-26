use std::mem;

pub use crate::error::Error;

const LZNT1_COMPRESSED_FLAG: usize = 0x8000;

macro_rules! load16le{
    ($dst:expr,$src:expr,$idx:expr)=>{
        {
            $dst = (u32::from($src[$idx + 1]) << 8
            | u32::from($src[$idx])) as usize;
        }
    }
}

pub fn decompress(
    in_buf: &[u8]
) -> Result<Vec<u8>, Error>
{
    
    let mut out_buf: Vec<u8> = Vec::with_capacity(in_buf.len());

    match decompress2_push(in_buf, &mut out_buf) {
        Err(e) => println!("{:?}", e),
        _ => ()
    }
    
    Ok(out_buf)
}

pub fn decompress2_push_old(
    in_buf: &[u8],
    out_buf: &mut Vec<u8>
) -> Result<(), Error>
{
    let mut out_idx: usize = 0;
    let mut in_idx:  usize = 0;

    let mut header:     usize;
    let mut length:     usize;
    let mut block_len:  usize;
    let mut offset:     usize;

    let mut _block_id = 0;
    while in_idx < in_buf.len() {
        let in_chunk_base = in_idx;
        load16le!(header, in_buf, in_idx);
        in_idx += mem::size_of::<u16>();
        block_len = (header & 0xfff) + 1;

        if block_len > (in_buf.len() - in_idx) {
            return Err(Error::MemLimit);
        } else {
            if header & LZNT1_COMPRESSED_FLAG != 0 {
                let in_base_idx = in_idx;
                let out_base_idx = out_idx;
                while (in_idx - in_base_idx) < block_len {
                    if in_idx >= in_buf.len() {
                        break;
                    }

                    let flags = in_buf[in_idx];
                    in_idx += mem::size_of::<u8>();

                    for n in 0..8 {
                        if ((flags >> n) & 1) == 0 {
                            if in_idx >= in_buf.len() || (in_idx - in_base_idx) >= block_len {
                                break;
                            }
                            out_buf.push(in_buf[in_idx]);
                            out_idx += mem::size_of::<u8>();
                            in_idx += mem::size_of::<u8>();
                        } else {
                            let flag;
                            if in_idx >= in_buf.len() || (in_idx - in_base_idx) >= block_len {
                                break;
                            }
                            load16le!(flag, in_buf, in_idx);
                            in_idx += mem::size_of::<u16>();

                            let mut pos = out_idx - out_base_idx - 1;
                            let mut l_mask = 0xFFF;
                            let mut o_shift = 12;
                            while pos >= 0x10 {
                                l_mask >>= 1;
                                o_shift -= 1;
                                pos >>= 1;
                            }

                            length = (flag & l_mask) + 3;
                            offset = (flag >> o_shift) + 1;
                            if length >= offset {
                                let count = (0xfff / offset) + 1;
                                if offset > out_idx {
                                    return Err(Error::CorruptedData);
                                }

                                let chunk_pos = out_idx - offset;
                                let chunk_len = offset;

                                let mut x = 0;
                                while x < length {
                                    for _i in 0..count {
                                        for _j in 0..chunk_len {
                                            out_buf.push(out_buf[chunk_pos + _j]);
                                            out_idx += mem::size_of::<u8>();
                                            x += 1;
                                            if x >= length {
                                                break;
                                            }
                                        }

                                        if x >= length {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                for _i in 0..length {
                                    if offset > out_idx {
                                        return Err(Error::CorruptedData);
                                    }
                                    out_buf.push(out_buf[out_idx - offset]);
                                    out_idx += mem::size_of::<u8>();
                                }
                            }
                        }
                    }
                }
            } else {
                // Not compressed
                for _i in 0..block_len {
                    out_buf.push(in_buf[in_idx]);
                    out_idx += mem::size_of::<u8>();
                    in_idx += mem::size_of::<u8>();
                }
            }
        }

        in_idx = in_chunk_base + 2 + block_len;
        _block_id += 1;
    }

    Ok(())
}


pub fn decompress2_push(
    in_buf: &[u8],
    out_buf: &mut Vec<u8>
) -> Result<(), Error>
{
    let mut out_idx: usize = 0;
    let mut in_idx:  usize = 0;

    let mut header:     usize;
    let mut length:     usize;
    let mut chunk_len:  usize;
    let mut offset:     usize;

    let mut _block_id = 0;

    // We don't want to compute those values at each round.
    let in_buf_max_size = in_buf.len();

    while in_idx < in_buf_max_size {
        let in_chunk_base = in_idx;
        // compressed chunk header (2 bytes)
        load16le!(header, in_buf, in_idx);
        in_idx += mem::size_of::<u16>();
        chunk_len = (header & 0xfff) + 1;

        if chunk_len > (in_buf_max_size - in_idx) {
            return Err(Error::MemLimit);
        }

        if header & LZNT1_COMPRESSED_FLAG != 0 {
            let in_base_idx = in_idx;
            let out_base_idx = out_idx;

            let mut flag_bit = 0;
            let mut flags = in_buf[in_idx];
            in_idx += mem::size_of::<u8>();

            while (in_idx - in_base_idx) < chunk_len {
                if in_idx >= in_buf_max_size {
                    break;
                }

                if (flags & (1 << flag_bit)) == 0 {
                    if in_idx >= in_buf_max_size || (in_idx - in_base_idx) >= chunk_len {
                        break;
                    }

                    out_buf.push(in_buf[in_idx]);
                    out_idx += mem::size_of::<u8>();
                    in_idx += mem::size_of::<u8>();
                } else {
                    let copy_token;

                    if in_idx >= in_buf_max_size || (in_idx - in_base_idx) >= chunk_len {
                        break;
                    }

                    load16le!(copy_token, in_buf, in_idx);
                    in_idx += mem::size_of::<u16>();

                    let mut pos = out_idx - out_base_idx - 1;
                    let mut l_mask = 0xFFF;
                    let mut o_shift = 12;

                    while pos >= 0x10 {
                        l_mask >>= 1;
                        o_shift -= 1;
                        pos >>= 1;
                    }

                    length = (copy_token & l_mask) + 3;
                    offset = (copy_token >> o_shift) + 1;

                    if offset > out_idx {
                        return Err(Error::CorruptedData);
                    }

                    /*
                    if (out_idx + length) >= out_buf_max_size {
                        length = out_buf_max_size - out_idx;
                    }
                    */
                    
                    for _i in 0..length {
                        if offset > out_idx {
                            return Err(Error::CorruptedData);
                        }

                        out_buf.push(out_buf[out_idx - offset]);
                        out_idx += mem::size_of::<u8>();
                    }
                }

                flag_bit = (flag_bit + 1) % 8;

                if flag_bit == 0 {
                    if (in_idx - in_base_idx) >= chunk_len { break; }
                    flags = in_buf[in_idx];
                    in_idx += mem::size_of::<u8>();
                }
            }
        } else {
            // Not compressed
            for _i in 0..chunk_len {
                out_buf.push(in_buf[in_idx]);
                out_idx += mem::size_of::<u8>();
                in_idx += mem::size_of::<u8>();
            }
        }

        in_idx = in_chunk_base + 2 + chunk_len;
        _block_id += 1;
    }

    Ok(())
}

pub fn decompress2_no_push(
    in_buf: &[u8],
    out_buf: &mut Vec<u8>
) -> Result<(), Error>
{
    let mut out_idx: usize = 0;
    let mut in_idx:  usize = 0;

    let mut header:     usize;
    let mut length:     usize;
    let mut chunk_len:  usize;
    let mut offset:     usize;

    let mut _block_id = 0;

    // We don't want to compute those values at each round.
    let in_buf_max_size = in_buf.len();
    let out_buf_max_size = out_buf.len();

    if out_buf_max_size == 0 || out_buf_max_size < in_buf_max_size {
        return Err(Error::MemLimit);
    }

    while in_idx < in_buf_max_size {
        let in_chunk_base = in_idx;
        // compressed chunk header (2 bytes)
        load16le!(header, in_buf, in_idx);
        in_idx += mem::size_of::<u16>();
        chunk_len = (header & 0xfff) + 1;

        if chunk_len > (in_buf_max_size - in_idx) {
            return Err(Error::MemLimit);
        }

        if header & LZNT1_COMPRESSED_FLAG != 0 {
            let in_base_idx = in_idx;
            let out_base_idx = out_idx;

            let mut flag_bit = 0;
            let mut flags = in_buf[in_idx];
            in_idx += mem::size_of::<u8>();

            while (in_idx - in_base_idx) < chunk_len {
                if in_idx >= in_buf_max_size {
                    break;
                }

                if (flags & (1 << flag_bit)) == 0 {
                    if in_idx >= in_buf_max_size || (in_idx - in_base_idx) >= chunk_len {
                        break;
                    }
                    // out_buf.push(in_buf[in_idx]);
                    if out_idx >= out_buf_max_size {
                        return Err(Error::MemLimit);
                    }

                    out_buf[out_idx] = in_buf[in_idx];
                    out_idx += mem::size_of::<u8>();
                    in_idx += mem::size_of::<u8>();
                } else {
                    let copy_token;

                    if in_idx >= in_buf_max_size || (in_idx - in_base_idx) >= chunk_len {
                        break;
                    }

                    load16le!(copy_token, in_buf, in_idx);
                    in_idx += mem::size_of::<u16>();

                    let mut pos = out_idx - out_base_idx - 1;
                    let mut l_mask = 0xFFF;
                    let mut o_shift = 12;

                    while pos >= 0x10 {
                        l_mask >>= 1;
                        o_shift -= 1;
                        pos >>= 1;
                    }

                    length = (copy_token & l_mask) + 3;
                    offset = (copy_token >> o_shift) + 1;

                    if offset > out_idx {
                        return Err(Error::CorruptedData);
                    }

                    if (out_idx + length) >= out_buf_max_size {
                        length = out_buf_max_size - out_idx;
                    }
                    
                    for _i in 0..length {
                        if offset > out_idx {
                            return Err(Error::CorruptedData);
                        }
                        // out_buf.push(out_buf[out_idx - offset]);
                        if out_idx >= out_buf_max_size {
                            return Err(Error::MemLimit);
                        }
                        out_buf[out_idx] = out_buf[out_idx - offset];
                        out_idx += mem::size_of::<u8>();
                    }
                }

                flag_bit = (flag_bit + 1) % 8;

                if flag_bit == 0 {
                    if (in_idx - in_base_idx) >= chunk_len { break; }
                    flags = in_buf[in_idx];
                    in_idx += mem::size_of::<u8>();
                }
            }
        } else {
            // Not compressed
            for _i in 0..chunk_len {
                // out_buf.push(in_buf[in_idx]);
                if out_idx >= out_buf_max_size {
                    return Err(Error::MemLimit);
                }
                out_buf[out_idx] = in_buf[in_idx];
                out_idx += mem::size_of::<u8>();
                in_idx += mem::size_of::<u8>();
            }
        }

        in_idx = in_chunk_base + 2 + chunk_len;
        _block_id += 1;
    }

    unsafe {
        out_buf.set_len(out_idx);
    }

    Ok(())
}