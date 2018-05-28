#![feature(stdsimd)]
#![feature(align_offset, test)]

extern crate stdsimd;

mod std_validation;
mod simd_validation;

pub use std_validation::run_utf8_validation as regular;
pub use simd_validation::run_utf8_validation as simd;

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct Utf8Error {
    valid_up_to: usize,
    error_len: Option<u8>,
}

pub fn is_ascii_estimate_simd(utf8_slice: &[u8]) -> bool{
    if utf8_slice.len() >= 8 {
        let partial_text = stdsimd::simd::u8x8::load_unaligned(&utf8_slice[..8]);
        partial_text.le(stdsimd::simd::u8x8::splat(0x7F)).all()
    }else if utf8_slice.len() >= 4 {
        let partial_text = stdsimd::simd::u8x4::load_unaligned(&utf8_slice[..4]);
        partial_text.le(stdsimd::simd::u8x4::splat(0x7F)).all()
    }else{
        utf8_slice[..8].iter().all(|&x| x > 0x7F)
    }
}