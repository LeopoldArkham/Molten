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
use Molten::items::*;

lazy_static! {
    static ref MAP: HashMap<&'static str, fn() -> TOMLDocument<'static>> = {
        let mut m = HashMap::new();
        m.insert("empty", empty as fn() -> TOMLDocument<'static>);
        m.insert("simple", simple as fn() -> TOMLDocument<'static>);
        m
    };
}

#[test_case("tests/reconstruction/empty.toml", "empty" :: "Empty")]
#[test_case("tests/reconstruction/simple.toml", "simple" :: "Simple")]
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

fn simple() -> TOMLDocument<'static> {
    let mut container = Container::new();
    let trivia = LineMeta::empty();

    let bool_k = Key::new("bool");
    let bool_v = Item::Bool {val: true, meta: trivia.clone()};
    let _ = container.append(bool_k, bool_v);

    let string_k = Key::new("string");
    let string_v = Item::Str {
        t: StringType::SLB,
        val: "Hello!",
        original: "Hello!",
        meta: trivia.clone()
    };
    let _ = container.append(string_k, string_v);

    let int_k = Key::new("int");
    let int_v = Item::Integer {
        val: 42,
        meta: trivia.clone(),
        raw: "42"
    };
    let _ = container.append(int_k, int_v);

    TOMLDocument(container)
}