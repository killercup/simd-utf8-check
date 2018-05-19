#![feature(align_offset, test)]

mod std_validation;
mod simd_validation;

pub use std_validation::run_utf8_validation as regular;
pub use simd_validation::run_utf8_validation as simd;

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct Utf8Error {
    valid_up_to: usize,
    error_len: Option<u8>,
}