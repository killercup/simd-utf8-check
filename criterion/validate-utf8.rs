//! Benchmarks written by bluss
//!
//! cf <https://gist.github.com/bluss/bf45e07e711238e22b7a>

extern crate simd_utf8_check;
extern crate encoding_rs as encoding;
extern crate is_utf8;

#[macro_use] extern crate criterion;
use criterion::{Benchmark, Criterion, Throughput};

macro_rules! bench {
    ($name:ident, $path:expr) => {
        fn $name(c: &mut Criterion) {
            let text = include_bytes!($path);

            c.bench(stringify!($name),
                Benchmark::new("std", move |b| b.iter(|| ::simd_utf8_check::regular(text)))
                    .with_function("simd", move |b| b.iter(|| ::simd_utf8_check::simd(text)))
                    .with_function("encoding_rs", move |b| b.iter(|| encoding::Encoding::utf8_valid_up_to(text)))
                    .with_function("is_utf8", move |b| b.iter(|| ::is_utf8::is_utf8(text)))
                    .with_function("is_utf8_hoehrmann", move |b| b.iter(|| ::is_utf8::is_utf8_hoehrmann(text)))
                    .throughput(Throughput::Bytes(text.len() as u32))
            );
        }
    };
}

bench!(ascii, "../tests/fixtures/long.txt");
bench!(mixed, "../tests/fixtures/mixed.txt");
bench!(mosty_ascii, "../tests/fixtures/mostly_ascii.txt");
bench!(cyr, "../tests/fixtures/long_cy.txt");
bench!(enwiki8, "../tests/fixtures/long_cy.txt");
bench!(jawik10, "../tests/fixtures/jawik10");
bench!(big10, "../tests/fixtures/big10");

criterion_group!(benches, ascii, mixed, mosty_ascii, cyr, enwiki8, jawik10, big10);
criterion_main!(benches);
