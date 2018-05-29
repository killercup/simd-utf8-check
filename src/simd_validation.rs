//! Original C implementation by
//!
//! cf. <https://github.com/lemire/fastvalidate-utf-8>
//! cf. <https://lemire.me/blog/2018/05/16/validating-utf-8-strings-using-as-little-as-0-7-cycles-per-byte/>

use std::arch::x86_64::*;

struct ProcessedUtfBytes {
    rawbytes: __m128i,
    low_nibbles: __m128i,
    high_nibbles: __m128i,
    counts: __m128i,
}

impl Default for ProcessedUtfBytes {
    fn default() -> Self {
        ProcessedUtfBytes {
            rawbytes: unsafe { _mm_setzero_si128() },
            low_nibbles: unsafe { _mm_setzero_si128() },
            high_nibbles: unsafe { _mm_setzero_si128() },
            counts: unsafe { _mm_setzero_si128() },
        }
    }
}

fn check_smaller_than0x_f4(current_bytes_unsigned: __m128i, has_error: &mut __m128i) {
    unsafe {
        _mm_or_si128(
            *has_error,
            _mm_cmpgt_epi8(current_bytes_unsigned, _mm_set1_epi8(0xF4 - 128)),
        );
    }
}

// Code originally contributed by Kendall Willets
fn check_continuation(
    high_nibbles: __m128i,
    counts: __m128i,
    previous_counts: __m128i,
    has_error: &mut __m128i,
) {
    unsafe {
        let right1 = _mm_alignr_epi8(counts, previous_counts, 16 - 1);
        let right2 = _mm_subs_epu8(
            _mm_alignr_epi8(counts, previous_counts, 16 - 2),
            _mm_set1_epi8(1),
        );
        let right3 = _mm_subs_epu8(
            _mm_alignr_epi8(counts, previous_counts, 16 - 3),
            _mm_set1_epi8(2),
        );

        let following = _mm_or_si128(_mm_or_si128(right1, right2), right3);

        let continuations = _mm_cmpgt_epi8(following, _mm_set1_epi8(0));
        let firsts = _mm_shuffle_epi8(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            _mm_setr_epi8(
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // 0xxx
                0, 0, 0, 0, // 10xx
                0xFF, 0xFF, // 110x
                0xFF, // 1110
                0xFF, // 1111
            ),
            high_nibbles,
        );
        *has_error = _mm_or_si128(*has_error, _mm_cmpeq_epi8(firsts, continuations));
    }
}

fn check_first_continuation_max3(
    current_bytes_unsigned: __m128i,
    off1_low_nibbles: __m128i,
    off1_high_nibbles: __m128i,
    has_error: &mut __m128i,
) {
    unsafe {
        // the next max only kicks in if the low nibble is d
        let selector = _mm_and_si128(
            off1_high_nibbles,
            _mm_cmpeq_epi8(off1_low_nibbles, _mm_set1_epi8(0xD)),
        );
        // the -128  is to compensate for the signed arithmetic (lack of
        // _mm_cmpgt_epu8)
        let nextmax = _mm_shuffle_epi8(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            _mm_setr_epi8(0xFF - 128, 0xFF - 128, 0xFF - 128, 0xFF - 128, 0xFF - 128,
                            0xFF - 128, 0xFF - 128, 0xFF - 128, // 0xxx (ASCII)
                            0xFF - 128, 0xFF - 128, 0xFF - 128,
                            0xFF - 128, // 10xx (continuation)
                            0xFF - 128, 0xFF - 128, // 110x
                            0x9F - 128, // 1110
                            0xFF - 128), // 1111, next should be 0
            selector,
        );

        *has_error = _mm_or_si128(*has_error, _mm_cmpgt_epi8(current_bytes_unsigned, nextmax));
    }
}

fn check_first_continuation_max4(
    current_bytes_unsigned: __m128i,
    off1_low_nibbles: __m128i,
    off1_high_nibbles: __m128i,
    has_error: &mut __m128i,
) {
    unsafe {
        // the next max only kicks in if the low nibble is 4

        let selector = _mm_and_si128(
            off1_high_nibbles,
            _mm_cmpeq_epi8(off1_low_nibbles, _mm_set1_epi8(0x4)),
        );
        // the -128  is to compensate for the signed arithmetic (lack of
        // _mm_cmpgt_epu8)
        let nextmax = _mm_shuffle_epi8(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            _mm_setr_epi8(0xFF - 128, 0xFF - 128, 0xFF - 128, 0xFF - 128, 0xFF - 128,
                0xFF - 128, 0xFF - 128, 0xFF - 128, // 0xxx (ASCII)
                0xFF - 128, 0xFF - 128, 0xFF - 128,
                0xFF - 128, // 10xx (continuation)
                0xFF - 128, 0xFF - 128, // 110x
                0xFF - 128, // 1110
                0x8F - 128, // 1111, next should be 0
            ),
            selector,
        );
        *has_error = _mm_or_si128(*has_error, _mm_cmpgt_epi8(current_bytes_unsigned, nextmax));
    }
}

fn check_first_continuation_min(
    current_bytes_unsigned: __m128i,
    off1_low_nibbles: __m128i,
    off1_high_nibbles: __m128i,
    has_error: &mut __m128i,
) {
    unsafe {
        let mut nextmin = _mm_shuffle_epi8(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            _mm_setr_epi8(
                0, 0, 0, 0, 0, 0, 0, 0, // 0xxx (ASCII)
                0, 0, 0, 0, // 10xx (continuation)
                0, 0, // 110x
                0xA0, // 1110
                0x90, // 1111,
            ),
            off1_high_nibbles,
        );

        // the  mins only kicks in if the low nibble is zero

        nextmin = _mm_and_si128(
            nextmin,
            _mm_cmpeq_epi8(off1_low_nibbles, _mm_setzero_si128()),
        );

        let nextmin_unsigned = _mm_sub_epi8(nextmin, _mm_set1_epi8(-128));

        *has_error = _mm_or_si128(
            *has_error,
            _mm_cmpgt_epi8(nextmin_unsigned, current_bytes_unsigned),
        );
    }
}

fn check_larger_than0x_c2(
    current_bytes_unsigned: __m128i,
    high_nibbles: __m128i,
    has_error: &mut __m128i,
) {
    let thismin = unsafe {
        _mm_shuffle_epi8(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            _mm_setr_epi8(
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xxx (ASCII)
                0x00, 0x00, 0x00, 0x00, // 10xx (continuation)
                0xC2_u8 as i8, 0x00, // 110x
                0x00, // 1110
                0x00, // 1111
            ),
            high_nibbles,
        )
    };
    *has_error =
        unsafe { _mm_or_si128(*has_error, _mm_cmpgt_epi8(thismin, current_bytes_unsigned)) };
}

fn count_nibbles(bytes: __m128i) -> ProcessedUtfBytes {
    let mut answer = ProcessedUtfBytes::default();
    answer.rawbytes = bytes;
    let nibble_mask = unsafe { _mm_set1_epi8(0x0F) };
    answer.low_nibbles = unsafe { _mm_and_si128(bytes, nibble_mask) };
    answer.high_nibbles = unsafe { _mm_and_si128(_mm_srli_epi16(bytes, 4), nibble_mask) };
    answer.counts = unsafe {
        _mm_shuffle_epi8(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            _mm_setr_epi8(
                0, 0, 0, 0, 0, 0, 0, 0, // 0xxx (ASCII)
                0, 0, 0, 0, // 10xx (continuation)
                1, 1, // 110x
                2, // 1110
                3, // 1111, next should be 0 (not checked here)
            ),
            answer.high_nibbles,
        )
    };

    answer
}

fn check_utf8_bytes(
    current_bytes: __m128i,
    previous: &ProcessedUtfBytes,
    has_error: &mut __m128i,
) -> ProcessedUtfBytes {
    let pb = count_nibbles(current_bytes);
    let current_bytes_unsigned = unsafe { _mm_sub_epi8(current_bytes, _mm_set1_epi8(-128)) };
    check_smaller_than0x_f4(current_bytes_unsigned, has_error);
    check_larger_than0x_c2(current_bytes_unsigned, pb.high_nibbles, has_error);
    check_continuation(pb.high_nibbles, pb.counts, previous.counts, has_error);

    let off1_low_nibbles = unsafe { _mm_alignr_epi8(pb.low_nibbles, previous.low_nibbles, 16 - 1) };
    let off1_high_nibbles =
        unsafe { _mm_alignr_epi8(pb.high_nibbles, previous.high_nibbles, 16 - 1) };
    check_first_continuation_max3(
        current_bytes_unsigned,
        off1_low_nibbles,
        off1_high_nibbles,
        has_error,
    );
    check_first_continuation_max4(
        current_bytes_unsigned,
        off1_low_nibbles,
        off1_high_nibbles,
        has_error,
    );
    check_first_continuation_min(
        current_bytes_unsigned,
        off1_low_nibbles,
        off1_high_nibbles,
        has_error,
    );
    pb
}

pub fn run_utf8_validation(src: &[u8]) -> bool {
    let len = src.len();
    let mut i = 0_usize;
    let mut has_error = unsafe { _mm_setzero_si128() };
    let mut previous = ProcessedUtfBytes::default();

    loop {
        if i + 15 >= len {
            break;
        }
        let current_bytes =
            unsafe { _mm_loadu_si128(src.as_ptr().offset(i as isize) as *const __m128i) };
        previous = check_utf8_bytes(current_bytes, &previous, &mut has_error);
        i += 16;
    }

    // last part
    if i < len {
        let mut buffer = [0_u8; 16];
        unsafe {
            ::std::ptr::copy_nonoverlapping(
                src.as_ptr().offset(i as isize),
                buffer.as_mut_ptr(),
                len - i,
            )
        };
        let current_bytes = unsafe { _mm_loadu_si128(buffer.as_ptr() as *const __m128i) };
        previous = check_utf8_bytes(current_bytes, &previous, &mut has_error);
    }

    unsafe { _mm_testz_si128(has_error, has_error) == 0 }
}
