//! Benchmarks written by bluss
//!
//! cf <https://gist.github.com/bluss/bf45e07e711238e22b7a>

extern crate test;

macro_rules! bench {
    ($name:ident, $fn:path, $path:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let text = ::std::fs::read_to_string($path).unwrap();
            b.iter(|| $fn(text.as_bytes()));
            b.bytes = text.len() as u64;
        }
    };
}

macro_rules! benches {
    ($fn:path) => {
        use super::test::Bencher;

        bench!(ascii, $fn, "tests/fixtures/long.txt");
        bench!(mixed, $fn, "tests/fixtures/mixed.txt");
        bench!(mosty_ascii, $fn, "tests/fixtures/mostly_ascii.txt");
        bench!(cyr, $fn, "tests/fixtures/long_cy.txt");
        bench!(enwiki8, $fn, "tests/fixtures/long_cy.txt");
        bench!(jawik10, $fn, "tests/fixtures/jawik10");
        bench!(big10, $fn, "tests/fixtures/big10");
    };
}

mod regular {
    benches!(super::super::regular);
}

mod simd {
    benches!(super::super::simd);
}
