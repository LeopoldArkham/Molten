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

    println!("{:#?}", res.0);
    // TODO: External "as_string()" must not contain quotes
    println!("{:?}",
             res["table"]["table.nested"]["table.nested.twice"][0].as_string());
    // @fixme: as_string function is inadequate for anything but internally
    // reproducing original input, as it omits the key of the value being displayed.
    assert_eq!(input, &res.as_string());
}
