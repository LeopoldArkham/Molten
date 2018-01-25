# Fuzzing Molten

This project is [fuzzed] using [cargo-fuzz] which is a very convenient tool that
makes using [libFuzzer] very easy.

## Huh?

Okay, that sentence was probably quite, errr, blurry, so let's rather describe
how to use this and why.

Since Molten is a parser (it reads TOML files), it operates on arbitrary input,
given by a user. It is a good idea to write Molten in such a way that it
doesn't crash on weird inputs, or, heaven forbid, leak data or put the system in
a corrupt state. That's where a fuzzer comes in: It runs (parts of) Molten with
seemingly random data. It then looks how Molten behaves and tries to adjust the
data it generates so that all the interesting code paths are triggered.

## Usage

First, install [cargo-fuzz] (like any other cargo subcommand) with

```bash
$ cargo install cargo-fuzz
```

Now, in the root directory of Molten, you can run

```bash
$ cargo fuzz list
```

to get a list of all available fuzzers. You can run one with:


```bash
$ cargo fuzz run parse
```

This runs the `parse` fuzzer, that lives in `fuzz/fuzz_targets/parse.rs`. It'll
compile the fuzzer as a special binary and then execute it with [libFuzzer].

To speed this up a bit, you can pass `--release`. To give parameters to libFuzzer,
do it like this:

```bash
$ cargo fuzz run parse --release -- -max_len=4100 -max_total_time=10
```

`-help=1` (as in `cargo fuzz run parse --release -- -help=1`) will give you a
list of available options.

[fuzzed]: https://en.wikipedia.org/wiki/Fuzzing
[cargo-fuzz]: https://github.com/rust-fuzz/cargo-fuzz
[libFuzzer]: https://releases.llvm.org/5.0.0/docs/LibFuzzer.html
