//! Parser benchmark tests to identify any performance regressions
//! in the parser.
#![feature(test)]
#![cfg(feature = "nightly")]

// For details on using Criterion, see the documentation at
// https://japaric.github.io/criterion.rs/book/index.html
extern crate criterion;
extern crate Molten;

use criterion::{Criterion, Bencher};

fn parse_full(b: &mut Bencher) -> () {
    use std::fs::File;
    use std::io::Read;

    let path = "tests/reproduction/full.toml";
    let mut content = String::new();
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut content).unwrap();

    b.iter(|| {
        let mut parser = Molten::parser::Parser::new(&content);
        parser.parse().unwrap();
    });
}

#[test]
fn parser_benchmark() {
    Criterion::default().bench_function("parser", parse_full);
}
