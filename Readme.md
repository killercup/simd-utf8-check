# SIMD UTF8 Validation in Rust

After reading the post [Validating UTF-8 strings using as little as 0.7 cycles per byte](https://lemire.me/blog/2018/05/16/validating-utf-8-strings-using-as-little-as-0-7-cycles-per-byte/),
I was curious if this algorithm might be a good fit for Rust's standard library.
Because Rust's `String` type is guaranteed to be UTF8,
you'll need to either use `from_utf8` to convert an array of bytes to a `String`,
or, if you trust the input, use the `unsafe fn from_utf8_unchecked`.
The faster `from_utf8` is, the more people can always use the safe version.

Of course, I'm not the first person to think of this,
and [this Rust PR](https://github.com/rust-lang/rust/pull/30740)
already contains a super fast implementation,
albeit one that that not use explicit SIMD intrinsics.

## Benchmarks

### Results

```
$ env RUSTFLAGS='-C target-cpu=native' cargo bench --quiet

running 12 tests
test benches::from_utf8_ascii_regular     ... bench:          87 ns/iter (+/- 24) = 29321 MB/s
test benches::from_utf8_ascii_simd        ... bench:         871 ns/iter (+/- 264) = 2928 MB/s
test benches::from_utf8_cyr_regular       ... bench:       6,791 ns/iter (+/- 2,017) = 755 MB/s
test benches::from_utf8_cyr_simd          ... bench:       1,796 ns/iter (+/- 323) = 2857 MB/s
test benches::from_utf8_enwik8_regular    ... bench:  12,755,057 ns/iter (+/- 1,489,178) = 7840 MB/s
test benches::from_utf8_enwik8_simd       ... bench:  33,230,547 ns/iter (+/- 3,653,408) = 3009 MB/s
test benches::from_utf8_jawik10_regular   ... bench: 275,079,207 ns/iter (+/- 10,982,357) = 743 MB/s
test benches::from_utf8_jawik10_simd      ... bench:  68,982,078 ns/iter (+/- 5,608,421) = 2965 MB/s
test benches::from_utf8_mixed_regular     ... bench:       2,101 ns/iter (+/- 503) = 2298 MB/s
test benches::from_utf8_mixed_simd        ... bench:       1,708 ns/iter (+/- 267) = 2827 MB/s
test benches::from_utf8_mostlyasc_regular ... bench:         216 ns/iter (+/- 44) = 16907 MB/s
test benches::from_utf8_mostlyasc_simd    ... bench:       1,224 ns/iter (+/- 303) = 2983 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 12 measured; 0 filtered out
```

(Measured on my late 2016 MacBook Pro with a Intel i7 6700HQ, and telling LLVM to use all features that this CPU supports.)

Looks like the current std impl is a bit faster for inputs that contain mostly ASCII,
but the SIMD version gives a significant speedup when dealing with multi-byte codepoints.

### Data

- jawik10: `curl -L http://dumps.wikimedia.org/archive/2006/2006-07/jawiki/20061016/jawiki-20061016-pages-articles.xml.bz2 | bunzip2 > test/fixtures/jawik10`
- enwiki8: From <http://mattmahoney.net/dc/textdata.html>
- `big10` is the dataset in <http://vaskir.blogspot.ru/2015/09/regular-expressions-rust-vs-f.html> (see <https://drive.google.com/open?id=0B8HLQUKik9VtUWlOaHJPdG0xbnM>)
