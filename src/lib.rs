#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]

#[macro_use]
extern crate pretty_assertions;
extern crate chrono;

mod tomlchar;
mod tomldoc;
mod parser;
mod index;
mod items;
mod container;
mod comment;

use std::fs::File;
use std::io::prelude::*;


#[test]
fn toml_test_1() {
    let input = include_str!("../toml_1.toml");
    let mut parser = parser::Parser::new(input);
    let res = parser.parse();

    let mut f = File::create("tomlres.toml").unwrap();
    let _ = f.write(res.as_string().as_bytes());

    // println!("{:#?}", res.0);
    println!("{:?}", res["table"]["broken_array"][0].as_string());
    assert_eq!(input, &res.as_string());
}