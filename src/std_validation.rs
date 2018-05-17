//! Impl from Rust's standard library

use std::mem;
use super::Utf8Error;

/// Walks through `iter` checking that it's a valid UTF-8 sequence,
/// returning `true` in that case, or, if it is invalid, `false` with
/// `iter` reset such that it is pointing at the first byte in the
/// invalid sequence.
#[inline]
pub fn run_utf8_validation(v: &[u8]) -> Result<(), Utf8Error> {
    let mut index = 0;
    let len = v.len();

    let usize_bytes = mem::size_of::<usize>();
    let ascii_block_size = 2 * usize_bytes;
    let blocks_end = if len >= ascii_block_size { len - ascii_block_size + 1 } else { 0 };

    while index < len {
        let old_offset = index;
        macro_rules! err {
            ($error_len: expr) => {
                return Err(Utf8Error {
                    valid_up_to: old_offset,
                    error_len: $error_len,
                })
            }
        }

        macro_rules! next { () => {{
            index += 1;
            // we needed data, but there was none: error!
            if index >= len {
                err!(None)
            }
            v[index]
        }}}

        let first = v[index];
        if first >= 128 {
            let w = UTF8_CHAR_WIDTH[first as usize];
            // 2-byte encoding is for codepoints  \u{0080} to  \u{07ff}
            //        first  C2 80        last DF BF
            // 3-byte encoding is for codepoints  \u{0800} to  \u{ffff}
            //        first  E0 A0 80     last EF BF BF
            //   excluding surrogates codepoints  \u{d800} to  \u{dfff}
            //               ED A0 80 to       ED BF BF
            // 4-byte encoding is for codepoints \u{1000}0 to \u{10ff}ff
            //        first  F0 90 80 80  last F4 8F BF BF
            //
            // Use the UTF-8 syntax from the RFC
            //
            // https://tools.ietf.org/html/rfc3629
            // UTF8-1      = %x00-7F
            // UTF8-2      = %xC2-DF UTF8-tail
            // UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
            //               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
            // UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
            //               %xF4 %x80-8F 2( UTF8-tail )
            match w {
                2 => if next!() & !CONT_MASK != TAG_CONT_U8 {
                    err!(Some(1))
                },
                3 => {
                    match (first, next!()) {
                        (0xE0         , 0xA0 ... 0xBF) |
                        (0xE1 ... 0xEC, 0x80 ... 0xBF) |
                        (0xED         , 0x80 ... 0x9F) |
                        (0xEE ... 0xEF, 0x80 ... 0xBF) => {}
                        _ => err!(Some(1))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(2))
                    }
                }
                4 => {
                    match (first, next!()) {
                        (0xF0         , 0x90 ... 0xBF) |
                        (0xF1 ... 0xF3, 0x80 ... 0xBF) |
                        (0xF4         , 0x80 ... 0x8F) => {}
                        _ => err!(Some(1))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(2))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(3))
                    }
                }
                _ => err!(Some(1))
            }
            index += 1;
        } else {
            // Ascii case, try to skip forward quickly.
            // When the pointer is aligned, read 2 words of data per iteration
            // until we find a word containing a non-ascii byte.
            let ptr = v.as_ptr();
            let align = unsafe {
                // the offset is safe, because `index` is guaranteed inbounds
                ptr.offset(index as isize).align_offset(usize_bytes)
            };
            if align == 0 {
                while index < blocks_end {
                    unsafe {
                        let block = ptr.offset(index as isize) as *const usize;
                        // break if there is a nonascii byte
                        let zu = contains_nonascii(*block);
                        let zv = contains_nonascii(*block.offset(1));
                        if zu | zv {
                            break;
                        }
                    }
                    index += ascii_block_size;
                }
                // step from the point where the wordwise loop stopped
                while index < len && v[index] < 128 {
                    index += 1;
                }
            } else {
                index += 1;
            }
        }
    }

    Ok(())
}

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte.
const TAG_CONT_U8: u8 = 0b1000_0000;

// use truncation to fit u64 into usize
const NONASCII_MASK: usize = 0x80808080_80808080u64 as usize;

/// Returns `true` if any byte in the word `x` is nonascii (>= 128).
#[inline]
fn contains_nonascii(x: usize) -> bool {
    (x & NONASCII_MASK) != 0
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];
