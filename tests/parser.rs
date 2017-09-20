#![feature(proc_macro)]

extern crate Molten;
extern crate test_case_derive;
extern crate pretty_assertions;


use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::fmt::Display;

use test_case_derive::test_case;

#[test_case("toml_1.toml" :: "toml_1")]
fn parser<P: AsRef<Path> + Display>(path: P) {
    let mut input = String::new();

    let mut f = File::open(&path).expect(&format!("Error opening: {}", path));
    f.read_to_string(&mut input).expect("Error reading file:");

    let mut parser = Molten::parser::Parser::new(&input);
    let res = parser.parse();

    assert_eq!(input, res.as_string());
}