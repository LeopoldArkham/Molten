#![feature(proc_macro)]

extern crate Molten;
extern crate test_case_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pretty_assertions;

#[allow(unused_imports)]
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;
use std::fmt::Display;
use std::collections::HashMap;

use test_case_derive::test_case;
use Molten::{TOMLDocument, Container};

lazy_static! {
    static ref MAP: HashMap<&'static str, fn() -> TOMLDocument<'static>> = {
        let mut m = HashMap::new();
        m.insert("empty", empty as fn() -> TOMLDocument<'static>);
        m
    };
}

#[test_case("tests/reconstruction/empty.toml", "empty" :: "Empty")]
// #[test_case("tests/reconstruction/simple.toml", simple :: "Simple")]
fn reconstuct<P: AsRef<Path> + Display>(path: P, constructor: &str) {
    let mut reference = String::new();
    let mut f = File::open(&path).unwrap();

    f.read_to_string(&mut reference).unwrap();

    assert_eq!(reference, MAP[constructor]().as_string());
}

fn empty() -> TOMLDocument<'static> {
    let container = Container::new();
    TOMLDocument(container)
}