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
# ...
$ open target/criterion/report/index.html
```

[You can also  find the rendered report here.](https://killercup.github.io/simd-utf8-check/report/index.html)
There are two runs, the first without and the second with the `target-cpu=native` flag.
This was benchmarked on a late 2016 MacBook Pro with an Intel i7 6700HQ CPU.

Currently, it looks like the current std impl is a bit faster for inputs that contain mostly ASCII,
but the SIMD version gives a significant speedup when dealing with multi-byte codepoints.

### Data

- jawik10: `curl -L http://dumps.wikimedia.org/archive/2006/2006-07/jawiki/20061016/jawiki-20061016-pages-articles.xml.bz2 | bunzip2 > test/fixtures/jawik10`
- enwiki8: From <http://mattmahoney.net/dc/textdata.html>
- `big10` is the dataset in <http://vaskir.blogspot.ru/2015/09/regular-expressions-rust-vs-f.html> (see <https://drive.google.com/open?id=0B8HLQUKik9VtUWlOaHJPdG0xbnM>)
