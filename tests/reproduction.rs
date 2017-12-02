#![allow(non_snake_case)]

extern crate Molten;
#[macro_use]
extern crate pretty_assertions;

#[allow(unused_imports)]
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;
use std::fmt::Display;

// To add a test case:
// 1) Create reference target toml file in /reproduction.
// 2) Add a matching test_case macro invocation to the list below.


/// Defines a test case for the `reproduction` test module. This macro takes
/// two arguments:
///
/// 1. `$path`: A relative path from the Cargo package's root directory to the TOML file
///    that should be parsed, and
/// 2. `$test_name`: An identifier that will be used as the test's name. Internally, it
///    becomes the name of the test function.
macro_rules! test_case {
    ($path:expr; $test_name:ident) => {
        #[test]
        fn $test_name() {
            reproduce($path);
        }
    };
}

test_case!("tests/reproduction/full.toml"; Full);
test_case!("tests/reproduction/integers.toml"; Integers);
test_case!("tests/reproduction/floats.toml"; Floats);
test_case!("tests/reproduction/bools.toml"; Bools);
test_case!("tests/reproduction/arrays.toml"; Arrays);
test_case!("tests/reproduction/comments.toml"; Comments);
test_case!("tests/reproduction/inline_tables.toml"; Inline_Tables);
test_case!("tests/reproduction/strings.toml"; Strings);
test_case!("tests/reproduction/tables.toml"; Tables);
test_case!("tests/reproduction/AoTs.toml"; AoTs);
test_case!("tests/reproduction/empty.toml"; Empty);
test_case!("tests/reproduction/whitespace.toml"; Whitespace);
test_case!("tests/reproduction/AoT_simple.toml"; AoT_Simple);
test_case!("tests/reproduction/quoted_keys.toml"; Quoted_Keys);
test_case!("tests/reproduction/kv_sep.toml"; Kv_Separators);

/// This tests the parser's correctness by parsing each of the
/// above files and attempting to reproduce them from the parsed structure.
/// Any difference between original and reproduction is a bug.
fn reproduce<P: AsRef<Path> + Display>(path: P) {
    let mut input = String::new();

    let mut f = File::open(&path).expect(&format!("Error opening: {}", path));
    f.read_to_string(&mut input).expect("Error reading file:");

    let mut parser = Molten::parser::Parser::new(&input);
    let res = parser.parse().unwrap();

    // Knobs for debugging
    // let mut f = File::create("tests/reproduction/res.txt").unwrap();
    // println!("{:#?}", res);
    // let _ = f.write(format!("{:#?}", res).as_bytes()).unwrap();

    assert_eq!(input, res.as_string());
}
