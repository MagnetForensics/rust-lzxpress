//
// C implementation of the LZNT1 algorithm.
// We call this function directly from the rust unit tests,
// and for the benchmark unit tests to compare the performance
// of running the function in C versus Rust.
//

#include <stdio.h>
#include <stdbool.h>

#define LZNT1_COMPRESSED_FLAG 0x8000

bool
decompress_lznt1(
    unsigned char *in_buf,
    int in_buf_max_size,
    unsigned char *out_buf,
    int out_buf_max_size,
    int *pout_buf_size
)
{
    int out_idx = 0;
    int in_idx = 0;

    unsigned short header = 0;
    int length = 0;
    int chunk_len = 0;
    int offset = 0;

    if ((out_buf_max_size == 0) || 
        (out_buf_max_size < in_buf_max_size)) {
        return false;
    }

    while (in_idx < in_buf_max_size) {
        int in_chunk_base = in_idx;
        // compressed chunk header (2 bytes)
        header = (in_buf[in_idx] | (in_buf[in_idx + 1] << 8));
        in_idx += sizeof(unsigned short);
        chunk_len = (header & 0xfff) + 1;

        if (chunk_len > (in_buf_max_size - in_idx)) {
            return false;
        }

        if ((header & LZNT1_COMPRESSED_FLAG) != 0) {
            // compressed chunk
            int in_base_idx = in_idx;
            int out_base_idx = out_idx;

            int flag_bit = 0;
            int flags = in_buf[in_idx];
            in_idx += sizeof(unsigned char);
            
            int format = 0;

            while ((in_idx - in_base_idx) < chunk_len) {

                if ((in_idx >= in_buf_max_size) ||
                    (out_idx >= out_buf_max_size)) {
                    break;
                }

                if ((flags & (1 << flag_bit)) == 0) {
                    // out_buf.push(in_buf[in_idx]);
                    if (out_idx >= out_buf_max_size) {                                              
                        printf("[!][%s:%s:%d] out_idx > out_buf_max_size\n",
                            __FILE__, __func__ , __LINE__);
                        printf("out_idx = 0x%x out_buf_max_size = 0x%x\n",
                            out_idx, out_buf_max_size);
                        return false;
                    }

                    out_buf[out_idx] = in_buf[in_idx];
                    out_idx += sizeof(unsigned char);
                    in_idx += sizeof(unsigned char);
                } else {
                    if ((in_idx >= in_buf_max_size) || 
                        ((in_idx - in_base_idx) >= chunk_len)) {
                        break;
                    }

                    int copy_token = (in_buf[in_idx] | (in_buf[in_idx + 1] << 8));
                    in_idx += sizeof(unsigned short);

                    int pos = out_idx - out_base_idx - 1;
                    int l_mask = 0xfff;
                    int o_shift = 12;

                    while (pos >= 0x10) {
                        l_mask >>= 1;
                        o_shift -= 1;
                        pos >>= 1;
                    }

                    length = (copy_token & l_mask) + 3;
                    offset = (copy_token >> o_shift) + 1;

                    // length = (copy_token & (l_mask - format)) + 3;
                    // offset = (copy_token >> (o_shift + format)) + 1;

                    if (offset > out_idx) {
                        printf("[!][%s:%s:%d] offset > out_idx",
                            __FILE__, __func__ , __LINE__);
                        return false;
                    }

                    if ((out_idx + length) >= out_buf_max_size) {
                        length = out_buf_max_size - out_idx;
                    }

                    for (int i = 0; i < length; i++) {
                        out_buf[out_idx] = out_buf[out_idx - offset];
                        out_idx += sizeof(unsigned char);
                    }
                }
                    
                flag_bit = (flag_bit + 1) % 8;

                if (!flag_bit) {
                    if ((in_idx - in_base_idx) >= chunk_len) break;
                    flags = in_buf[in_idx];
                    in_idx += sizeof(unsigned char);
                }
            }
        } else {
            // Not compressed
            if ((out_idx + chunk_len) >= out_buf_max_size) {
                printf("[!][%s:%s:%d] out_idx > out_buf_max_size",
                    __FILE__, __func__ , __LINE__);
                return false;
            }

            for (int i = 0; i < chunk_len; i++) {
                out_buf[out_idx] = in_buf[in_idx];
                out_idx += sizeof(unsigned char);
                in_idx += sizeof(unsigned char);
            }
        }

        in_idx = in_chunk_base + 2 + chunk_len;
    }

    *pout_buf_size = out_idx;
    return true;
}