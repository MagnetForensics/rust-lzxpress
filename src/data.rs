use std::mem;

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