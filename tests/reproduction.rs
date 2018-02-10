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
// 2) Add a matching `test_case!` macro invocation to the list below.


/// Defines a test case for the `reproduction` test module. This macro takes
/// three arguments:
///
/// 1. `$path`: A relative path from the Cargo package's root directory to the TOML file
///    that should be parsed,
/// 2. `$debug`: A boolean to specify whether the internals of the parsed document should
///    be printed out to the terminal, and
/// 2. `$test_name`: An identifier that will be used as the test's name. Internally, it
///    becomes the name of the test function.
macro_rules! test_case {
    ($path:expr, $debug:expr; $test_name:ident) => {
        #[test]
        fn $test_name() {
            reproduce($path, $debug);
        }
    };
}

test_case!("tests/reproduction/full.toml", false; Full);
test_case!("tests/reproduction/integers.toml", false; Integers);
test_case!("tests/reproduction/floats.toml", false; Floats);
test_case!("tests/reproduction/bools.toml", false; Bools);
test_case!("tests/reproduction/arrays.toml", false; Arrays);
test_case!("tests/reproduction/comments.toml", false; Comments);
test_case!("tests/reproduction/inline_tables.toml", false; Inline_Tables);
test_case!("tests/reproduction/strings.toml", false; Strings);
test_case!("tests/reproduction/tables.toml", false; Tables);
test_case!("tests/reproduction/AoTs.toml", false; AoTs);
test_case!("tests/reproduction/broken_up_AoT.toml", true; Broken_Up_AoT);
test_case!("tests/reproduction/empty.toml", false; Empty);
test_case!("tests/reproduction/whitespace.toml", false; Whitespace);
test_case!("tests/reproduction/AoT_simple.toml", false; AoT_Simple);
test_case!("tests/reproduction/quoted_keys.toml", false; Quoted_Keys);
test_case!("tests/reproduction/kv_sep.toml", false; Kv_Separators);

/// This tests the parser's correctness by parsing each of the
/// above files and attempting to reproduce them from the parsed structure.
/// Any difference between original and reproduction is a bug.
fn reproduce<P: AsRef<Path> + Display>(path: P, debug: bool) {
    let mut input = String::new();

    let mut f = File::open(&path).expect(&format!("Error opening: {}", path));
    f.read_to_string(&mut input).expect("Error reading file:");

    let mut parser = Molten::parser::Parser::new(&input);
    let res = parser.parse().unwrap();

    if debug {
        let mut f = File::create("tests/reproduction/res.txt").unwrap();
        let _ = f.write(format!("{:#?}", res).as_bytes()).unwrap();
        println!("{:#?}", res);
    }

    assert_eq!(input, res.as_string());
}
