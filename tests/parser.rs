#![feature(proc_macro)]
#![allow(non_snake_case)]

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

// #[test_case("tests/full.toml" :: "Full")]
// #[test_case("tests/integers.toml" :: "Integers")]
// #[test_case("tests/floats.toml" :: "Floats")]
// #[test_case("tests/bools.toml" :: "Bools")]
// #[test_case("tests/arrays.toml" :: "Arrays")]
// #[test_case("tests/comments.toml" :: "Comments")]
// #[test_case("tests/inline_tables.toml" :: "Inline Tables")]
// #[test_case("tests/strings.toml" :: "Strings")]
// #[test_case("tests/tables.toml" :: "Tables")]
#[test_case("tests/AoTs.toml" :: "AoT's")]
// #[test_case("tests/empty.toml" :: "Empty")]
// #[test_case("tests/whitespace.toml" :: "Whitespace")]
fn parser<P: AsRef<Path> + Display>(path: P) {
    let mut input = String::new();

    let mut f = File::open(&path)
        .expect(&format!("Error opening: {}", path));
    f.read_to_string(&mut input).expect("Error reading file:");

    let mut parser = Molten::parser::Parser::new(&input);
    let res = parser.parse();

    let mut f = File::create("tests/res.toml").unwrap();
    // println!("{:#?}", res);
    let _ = f.write(res.as_string().as_bytes()).unwrap();

    assert_eq!(input, res.as_string());
}