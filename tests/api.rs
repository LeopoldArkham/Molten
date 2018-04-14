extern crate Molten;
#[macro_use]
extern crate pretty_assertions;

use std::io::Read;
use std::fs::File;

use Molten::key_value;

#[test]
fn newline_before_append() {
    let kv = format!("{}{}", "test = 1", Molten::NL);
    let new_line = key_value(&kv).unwrap();
    
    let mut f = File::open("tests/api/newline_before_append.toml").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    let mut parser = Molten::parser::Parser::new(&buf);
    let mut parsed = parser.parse().unwrap();

    parsed.append_nl_check(new_line.0, new_line.1).unwrap();

    let reference = {
        let mut f = File::open("tests/api/newline_before_append_reference.toml").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        buf
    };

    assert_eq!(parsed.as_string(), reference);
    std::mem::drop(parsed);
}