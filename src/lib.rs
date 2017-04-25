#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]

#[macro_use]
extern crate pretty_assertions;
extern crate linked_hash_map;
extern crate chrono;

mod tomlchar;
mod tomldoc;
mod parser;
mod TOMLElements;

use std::str::FromStr;
use std::fs::File;
use std::io::prelude::*;

use linked_hash_map::LinkedHashMap;



// TODO: All string types
// TODO: Replace TLV mess with generic trait objects over flat 'item' space.
// -> 
// TODO: Move to cow and check for implicit string copies
// TODO: Better, separate test infrastructure
// TODO: Debug view &|| logging

#[test]
fn toml_test_1() {
    let input = include_str!("../toml_1.toml");
    let mut parser = parser::Parser::new(input);
    let res = parser.parse();

    let mut f = File::create("tomlres.toml").unwrap();
    let _ = f.write(res.as_string().as_bytes());

    println!("{:#?}", res.0);
    assert_eq!(input, &res.as_string());
}
