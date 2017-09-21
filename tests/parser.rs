#![feature(proc_macro)]

extern crate Molten;
extern crate test_case_derive;
#[macro_use]
extern crate pretty_assertions;

#[allow(unused_imports)]
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;
use std::fmt::Display;

use test_case_derive::test_case;

// @incomplete: Full array compliance
// @incomplete: Comments in arrays

#[test_case("tests/full.toml" :: "Full")]
#[test_case("tests/integers.toml" :: "Integers")]
#[test_case("tests/floats.toml" :: "Floats")]
#[test_case("tests/bools.toml" :: "Bools")]
#[test_case("tests/arrays.toml" :: "Arrays")]
#[test_case("tests/comments.toml" :: "Comments")]
fn parser<P: AsRef<Path> + Display>(path: P) {
    let mut input = String::new();

    let mut f = File::open(&path).expect(&format!("Error opening: {}", path));
    f.read_to_string(&mut input).expect("Error reading file:");

    let mut parser = Molten::parser::Parser::new(&input);
    let res = parser.parse();

    let mut f = File::create("tests/res.toml").unwrap();
    f.write(res.as_string().as_bytes()).unwrap();

    assert_eq!(input, res.as_string());
}