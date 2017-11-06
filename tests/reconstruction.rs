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
        m.insert("AoTs", AoTs as fn() -> TOMLDocument<'static>);
        m
    };
}

#[test_case("tests/reconstruction/empty.toml", "empty" :: "Empty")]
#[test_case("tests/reconstruction/simple.toml", "simple" :: "Simple")]
#[test_case("tests/reconstruction/AoTs.toml", "AoTs" :: "AoT's")]
/// Constructs a copy of the reference document using the API and 
/// compares the two TOMLDoc hierarchies.
fn reconstuct<P: AsRef<Path> + Display>(path: P, constructor: &str) {
    let mut reference = String::new();
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut reference).unwrap();

    let under_test = MAP[constructor]();
    let parsed = {
        let mut parser = Molten::parser::Parser::new(&reference);
        parser.parse().unwrap()
    };

    assert_eq!(reference, under_test.as_string());
    assert_eq!(parsed, under_test);
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
    let int_v = Item::integer("42");
    let _ = container.append(int_k, int_v);

    TOMLDocument(container)
}

fn AoTs() -> TOMLDocument<'static> {
    let mut container = Container::new();
    let trivia = LineMeta::empty();

    let mut payload_first = Vec::new();

    let first_1 = {
        let mut _container = Container::new();
        let id_k = Key::new("id");
        let id_v = Item::integer("1");
        let _ = _container.append(id_k, id_v);

        let nested_id_k = Key::new("nested_id");
        let nested_id_v = Item::integer("12");

        let mut nested_container = Container::new();
        let _ = nested_container.append(nested_id_k, nested_id_v);
        let _ = nested_container.append(None, Item::WS("\n"));

        let nested_k = Key::new("first.nested");
        let nested_v = Item::Table {
            is_array: false,
            val: nested_container,
            meta: trivia.clone(),
        };

        let _ = _container.append(nested_k, nested_v);

        Item::Table {
            is_array: true,
            val: _container,
            meta: trivia.clone(),
        }
    };

    let first_2 = {
        let mut _container = Container::new();
        let id_k = Key::new("id");
        let id_v = Item::integer("2");
        let _ = _container.append(id_k, id_v);
        let _ = _container.append(None, Item::WS("\n"));

        Item::Table {
            is_array: true,
            val: _container,
            meta: trivia.clone(),
        }
    };

    let first_3 = {
        let mut _container = Container::new();
        let id_k = Key::new("id");
        let id_v = Item::integer("3");
        let _ = _container.append(id_k, id_v);

        let nested_id_k = Key::new("nested_id");
        let nested_id_v = Item::integer("31");

        let mut _payload = Vec::new();

        let boolean_k = Key::new("bool");
        let boolean_v = Item::Bool {val: true, meta: trivia.clone()};
        let mut table_container = Container::new();
        let _ = table_container.append(boolean_k, boolean_v);
        let _ = table_container.append(None, Item::WS("\n"));


        let table = Item::Table {
            is_array: true,
            val: table_container,
            meta: trivia.clone(),
        };
        _payload.push(table.clone());
        _payload.push(table.clone());

        let nestedagain_k = Key::new("first.nested.nestedagain");
        let nestedagain_v = Item::AoT(_payload);

        let mut nested_container = Container::new();
        let _ = nested_container.append(nested_id_k, nested_id_v);
        let _ = nested_container.append(nestedagain_k, nestedagain_v);

        let nested_k = Key::new("first.nested");
        let nested_v = Item::Table {
            is_array: false,
            val: nested_container,
            meta: trivia.clone(),
        };

        let _ = _container.append(nested_k, nested_v);

        Item::Table {
            is_array: true,
            val: _container,
            meta: trivia.clone(),
        }
    };

    payload_first.push(first_1);
    payload_first.push(first_2);
    payload_first.push(first_3);
    
    let first_k = Key::new("first");
    let first_v = Item::AoT(payload_first);

    let mut payload_second = Vec::new();
    let table = Item::Table {
        is_array: true,
        val: Container::new(),
        meta: trivia.clone(),
    };
    payload_second.push(table.clone());
    payload_second.push(table.clone());
    payload_second.push(table.clone());    

    let second_k = Key::new("second");
    let second_v = Item::AoT(payload_second);


    let _ = container.append(first_k, first_v);
    let _ = container.append(second_k, second_v);
    
    TOMLDocument(container)
}