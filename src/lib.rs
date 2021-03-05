pub mod error;

pub use error::LzxpressError;

use std::str;

pub fn decompress(
    input_buffer: &[u8]
) -> Result<Vec<u8>, LzxpressError>
{
    let mut output_index: usize = 0;
    let mut input_index: usize = 0;

    let mut indicator: u32 = 0;
    let mut indicator_bit: u32 = 0;
    let mut length: usize;

    let mut nibble_index: usize = 0;

    let mut offset: usize;

    let mut output_buffer: Vec<u8> = Vec::new();

    loop {
        if input_index >= input_buffer.len() {
            break;
            // return Err(LzxpressError::MemLimit);
        }

        if indicator_bit == 0
        {
            indicator = u32::from(input_buffer[input_index + 3]) << 24
                        | u32::from(input_buffer[input_index + 2]) << 16
                        | u32::from(input_buffer[input_index + 1]) << 8
                        | u32::from(input_buffer[input_index]);

            input_index = input_index + 4; // sizeof(uint32_t);

            indicator_bit = 32;
        }

        indicator_bit = indicator_bit - 1;

        //
        // Check whether the bit specified by indicator_bit is set or not 
        // set in indicator. For example, if indicator_bit has value 4 
        // check whether the 4th bit of the value in indicator is set.
        //

        if ((indicator >> indicator_bit) & 1) == 0
        {
            if input_index >= input_buffer.len() {
                break; // return Err(LzxpressError::MemLimit);
            }

            output_buffer.push(input_buffer[input_index]);

            input_index += 1; // sizeof(uint8_t);
            output_index += 1; // sizeof(uint8_t);
        }
        else
        {
            if (input_index + 1) >= input_buffer.len() { 
                break; // return Err(LzxpressError::MemLimit);
            }

            length = usize::from(input_buffer[input_index + 1]) << 8
                    | usize::from(input_buffer[input_index]);

            input_index = input_index + 2; // sizeof(uint16_t);

            offset = length / 8;
            length = length % 8;

            if length == 7
            {
                if nibble_index == 0
                {
                    nibble_index = input_index;

                    if input_index >= input_buffer.len() {
                        break; // return Err(LzxpressError::MemLimit);
                    }

                    length = (input_buffer[input_index] % 16).into();

                    input_index = input_index + 1; // sizeof(uint8_t);
                }
                else
                {
                    if nibble_index >= input_buffer.len() {
                        break; // return Err(LzxpressError::MemLimit);
                    }

                    length = (input_buffer[nibble_index] / 16).into();
                    nibble_index = 0;
                }

                if length == 15
                {
                    if input_index >= input_buffer.len() {
                        break; // return Err(LzxpressError::MemLimit);
                    }

                    length = input_buffer[input_index].into();

                    input_index = input_index + 1; // sizeof(uint8_t);

                    if length == 255
                    {
                        if (input_index + 1) >= input_buffer.len() {
                            break; // return Err(LzxpressError::MemLimit);
                        }

                        length = usize::from(input_buffer[input_index + 1]) << 8
                                | usize::from(input_buffer[input_index]);

                        input_index = input_index + 2; // sizeof(uint16_t);

                        length = length - (15 + 7);
                    }
                    length = length + 15;
                }
                length = length + 7;
            }

            length = length + 3;

            while length != 0
            {
                if ((offset + 1) > output_index) {
                    break;
                }

                output_buffer.push(output_buffer[output_index - offset - 1]);

                output_index = output_index + 1; //sizeof(uint8_t);
                length = length - 1; // sizeof(uint8_t);
            }
        }
    }

    return Ok(output_buffer);
}