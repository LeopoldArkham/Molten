#![allow(non_snake_case)]

use std::fs::File;
use std::io::Read;

extern crate Molten;
#[macro_use]
extern crate pretty_assertions;

#[test]
fn simple() {
    let mut f = File::open("tests/indexing/simple.toml").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let parsed = {
        let mut parser = Molten::parser::Parser::new(&buf);
        parser.parse().unwrap()
    };

    assert_eq!(parsed[0], parsed["val1"]);
    assert_eq!(parsed[1], parsed["val2"]);
    assert_eq!(parsed[2], parsed["val3"]);
}

#[test]
fn skip_trivia() {
    let mut f = File::open("tests/indexing/skip_trivia.toml").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let parsed = {
        let mut parser = Molten::parser::Parser::new(&buf);
        parser.parse().unwrap()
    };

    assert!(parsed[0].is_value());
    assert!(parsed[1].is_value());
    assert!(parsed[2].is_value());
    assert!(parsed[3].is_value());    

    assert!(parsed[3][0].is_value());
    assert!(parsed[3][1].is_value());
    assert!(parsed[3][2].is_value());
    assert!(parsed[3][3].is_value());
}

#[test]
fn Aot_simple() {
    let mut f = File::open("tests/indexing/AoT_simple.toml").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let parsed = {
        let mut parser = Molten::parser::Parser::new(&buf);
        parser.parse().unwrap()
    };

    assert!(parsed.iter().count() == 1);
    for i in 0..2 {
        assert!(parsed[0][i]["value"].as_integer().unwrap() == i as i64);
    }
}

#[test]
fn Aot_broken_up() {
    let mut f = File::open("tests/indexing/broken_up_AoT.toml").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let parsed = {
        let mut parser = Molten::parser::Parser::new(&buf);
        parser.parse().unwrap()
    };

    for i in 0..2 {
        assert!(parsed[0][i]["value"].as_integer().unwrap() == i as i64);
    }
}