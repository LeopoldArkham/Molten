#![allow(non_snake_case)]

#[macro_use]
extern crate Molten;
#[macro_use]
extern crate pretty_assertions;

//#[allow(unused_imports)]
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;
use std::fmt::Display;

// @cleanup: integer scope
use Molten::{TOMLDocument, Container, integer};
use Molten::errors::*;
use Molten::items::*;
use Molten::NL;

// To add a test case:
// 1) Write a target toml file in /reconstruction
// 2) Write a public function with the same name inside the `constructors` module (in this file),
// which programatically reproduces your target document.
// 3) Add a test_case macro invocation for your new test to the list right above that module.


/// Defines a test case for the `reconstruction` test module. This macro takes
/// three arguments:
///
/// 1. `$path`: A relative path from the Cargo package's root directory to the TOML file
///    that should be parsed, and
/// 2. `$function`: An identifier referring to the test function that will be called. The
///    function must be defined with type `fn() -> TOMLDocument<'static>`, and it should
///    **not** have the `#[test]` attribute. It should also be defined in the `constructors`
///    module, and `constructors::` should be left off.
/// 3. `$test_name`: An identifier that will be used as the test's name. Internally, it
///    becomes the name of the function that does have the #[test] attribute.
macro_rules! test_case {
    ($path:expr, $function: ident; $test_name: ident) => {
        #[test]
        fn $test_name() {
            reconstruct($path, constructors::$function);
        }
    };
}


test_case!("tests/reconstruction/empty.toml", empty; Empty);
test_case!("tests/reconstruction/simple.toml", simple; Simple);
test_case!("tests/reconstruction/AoTs.toml", AoTs; AoTs);
test_case!("tests/reconstruction/whitespace.toml", whitespace; Whitespace);
test_case!("tests/reconstruction/indented.toml", indented; Indented);

mod constructors {
    use super::*;

    pub fn empty() -> Result<TOMLDocument<'static>> {
        let container = Container::new();
        Ok(TOMLDocument(container))
    }

    pub fn simple() -> Result<TOMLDocument<'static>> {
        let mut container = Container::new();
        let trivia = Trivia::empty();

        let bool_k = Key::new("bool");
        let bool_v = Item::Bool {
            val: true,
            meta: trivia.clone(),
        };
        let _ = container.append(bool_k, bool_v);

        let string_k = Key::new("string");
        let string_v = Item::Str {
            t: StringType::SLB,
            val: "Hello!",
            original: "Hello!",
            meta: trivia.clone(),
        };
        let _ = container.append(string_k, string_v);

        let int_k = Key::new("int");
        let int_v = integer("42")?;
        let _ = container.append(int_k, int_v);

        Ok(TOMLDocument(container))
    }

    pub fn AoTs() -> Result<TOMLDocument<'static>> {
        let mut container = Container::new();
        let trivia = Trivia::empty();

        let mut payload_first = Vec::new();

        let first_1 = {
            let mut _container = Container::new();
            let id_k = Key::new("id");
            let id_v = integer("1")?;

            let _ = _container.append(id_k, id_v);

            let nested_id_k = Key::new("nested_id");
            let nested_id_v = integer("12")?;

            let mut nested_container = Container::new();
            let _ = nested_container.append(nested_id_k, nested_id_v);
            let _ = nested_container.append(None, Item::WS(::NL));

            let nested_k = Key::new("first.nested");
            let nested_v = Item::Table {
                is_aot_elem: false,
                val: nested_container,
                meta: trivia.clone(),
            };

            let _ = _container.append(nested_k, nested_v);

            Item::Table {
                is_aot_elem: true,
                val: _container,
                meta: trivia.clone(),
            }
        };

        let first_2 = {
            let mut _container = Container::new();
            let id_k = Key::new("id");
            let id_v = integer("2")?;
            let _ = _container.append(id_k, id_v);
            let _ = _container.append(None, Item::WS(::NL));

            Item::Table {
                is_aot_elem: true,
                val: _container,
                meta: trivia.clone(),
            }
        };

        let first_3 = {
            let mut _container = Container::new();
            let id_k = Key::new("id");
            let id_v = integer("3")?;
            let _ = _container.append(id_k, id_v);

            let nested_id_k = Key::new("nested_id");
            let nested_id_v = integer("31")?;

            let mut _payload = Vec::new();

            let boolean_k = Key::new("bool");
            let boolean_v = Item::Bool {
                val: true,
                meta: trivia.clone(),
            };
            let mut table_container = Container::new();
            let _ = table_container.append(boolean_k, boolean_v);
            let _ = table_container.append(None, Item::WS(::NL));


            let table = Item::Table {
                is_aot_elem: true,
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
                is_aot_elem: false,
                val: nested_container,
                meta: trivia.clone(),
            };

            let _ = _container.append(nested_k, nested_v);

            Item::Table {
                is_aot_elem: true,
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
            is_aot_elem: true,
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

        Ok(TOMLDocument(container))
    }

    pub fn whitespace() -> Result<TOMLDocument<'static>> {
        let mut container = Container::new();
        let _trivia = Trivia::empty();
        let item = Item::WS(concat!(
            "           ",
            nl!(),
            "\t",
            nl!(),
            nl!(),
            "    ",
            nl!(),
            "  \t    ",
            nl!()
        ));
        container.append(None, item)?;
        Ok(TOMLDocument(container))
    }

    pub fn indented() -> Result<TOMLDocument<'static>> {
        let mut container = Container::new();

        let mut trivia = Trivia::empty();
        trivia.trail = concat!("  ", nl!());
        let key = Key::new("bool");
        let value = Item::Bool {
            val: true,
            meta: trivia,
        };
        container.append(key, value).unwrap();

        let mut trivia = Trivia::empty();
        trivia.indent = "\t";
        trivia.trail = concat!("\t", nl!());
        let key = Key::new("string");
        let value = Item::Str {
            t: StringType::SLB,
            val: "Hello!",
            original: "Hello!",
            meta: trivia,
        };
        container.append(key, value).unwrap();

        let _trivia = Trivia::empty();
        let value = Item::WS(concat!(nl!(), nl!()));
        container.append(None, value).unwrap();

        let mut trivia = Trivia::empty();
        trivia.indent = " ";
        let key = Key::new("int");
        let value = Item::Integer {
            val: 42,
            meta: trivia,
            raw: "42",
        };
        container.append(key, value).unwrap();

        Ok(TOMLDocument(container))
    }
}

/// Constructs a copy of the reference document using the API and
/// compares the two `TOMLDocument` hierarchies.
fn reconstruct<P: AsRef<Path> + Display>(path: P, under_test: fn() -> Result<TOMLDocument<'static>>) {
    let mut reference = String::new();
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut reference).unwrap();

    let parsed = {
        let mut parser = Molten::parser::Parser::new(&reference);
        parser.parse().unwrap()
    };

    let result = under_test().unwrap();
    assert_eq!(reference, result.as_string());
    assert_eq!(parsed, result);

    let mut original = File::create("parsed.txt").unwrap();
    let mut reconstructed = File::create("reconstructed.txt").unwrap();

    let _ = original.write(format!("{:#?}", parsed).as_bytes()).unwrap();
    let _ = reconstructed
        .write(format!("{:#?}", under_test).as_bytes())
        .unwrap();
}
