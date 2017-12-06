//! Parser benchmark tests to identify any performance regressions
//! in the parser.

// For details on using Criterion, see the documentation at
// https://japaric.github.io/criterion.rs/book/index.html
extern crate criterion;
#[macro_use()]
extern crate Molten;

use criterion::{Criterion, Bencher};
use std::fs::File;
use std::io::Read;

fn parse_indented(b: &mut Bencher) -> () {
    let path = "tests/reconstruction/indented.toml";
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
    Criterion::default().bench_function("parser", parse_indented);
}
