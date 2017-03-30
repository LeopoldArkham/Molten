#![allow(dead_code, non_snake_case)]

extern crate linked_hash_map;
extern crate chrono;

mod tomlchar;
mod tomldoc;
mod parser;
mod value;



use std::str::FromStr;

use linked_hash_map::LinkedHashMap;

use tomlchar::TOMLChar;
use value::Value;
use parser::Parser;

// TODO: All string types
// TODO: Stateful parser
// - Internal index
// - mark() function
// - mark-to-idx function
// TODO: Move to cow
// TODO: Separate tests
// TODO: Eat whitespace
// TODO: Debug view of idx positions
// TODO: Add logging


#[test]
fn toml_test_1() {
    let input = include_str!("../toml_1.txt");
    let mut parser = Parser::new(input);

    let res = parser.parse();
    println!("{:?}", res);
}

#[derive(Debug, PartialEq)]
pub struct Table {
    name: Vec<String>,
    comment: String,
    values: Vec<KeyValue>,
}

#[derive(Debug, PartialEq)]
pub enum Key {
    Bare(String),
    Quoted(String),
}

impl Key {
    // Useful?
    fn bare<T: Into<String>>(name: T) -> Key {
        Key::Bare(name.into())
    }

    fn quoted<T: Into<String>>(name: T) -> Key {
        Key::Quoted(name.into())
    }
}

#[derive(Debug, PartialEq)]
pub struct Comment {
    indent: String,
    comment: String,
}

#[derive(Debug, PartialEq)]
pub struct KeyValue {
    indent: String,
    key: Key,
    value: Value,
    comment: Option<Comment>,
}
// TESTS

// #[test]
// fn table_easy() {
//     let input = include_str!("../easy_table.toml");
//     let input = input.to_string().chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = Table {
//         name: vec!["test".to_string()],
//         comment: "".to_string(),
//         values: vec![
//         KeyValue{indent: "".to_string(),
//                  key: Key::Bare("myInt".to_string()),
//                  value: Value::Integer(5),
//                  comment: None},
//         KeyValue{indent: "".to_string(),
//                  key: Key::Bare("myString".to_string()),
//                  value: Value::SString("Hello".to_string()),
//                  comment: None},
//         KeyValue{indent: "".to_string(),
//                  key: Key::Bare("myBool".to_string()),
//                  value: Value::Bool(false),
//                  comment: None},
//     ],
//     };

//     assert_eq!(correct, parse_table(&input, &mut idx));
// }

// #[test]
// fn t_remove_brackets() {
//     let mut idx = 0;
//     let input = "[Section]".to_string().chars().collect::<Vec<char>>();
//     let r = remove_brackets(&input, &mut idx).iter().map(|c| *c).collect::<String>();
//     assert_eq!("Section", &r);

//     idx = 0;
//     let input = "[section.nested]".to_string().chars().collect::<Vec<char>>();
//     let r = remove_brackets(&input, &mut idx).iter().map(|c| *c).collect::<String>();
//     assert_eq!("section.nested", &r);
// }

// #[test]
// fn parse_inner_easy() {
//     let input = "section.nested".to_string().chars().collect::<Vec<char>>();
//     let r = parse_section_title(&input);
//     assert_eq!(["section".to_string(), "nested".to_string()], r.as_slice());
// }

// #[test]
// fn parse_inner_hard() {
//     let input = "section.\"pretty.hard\".nested".to_string().chars().collect::<Vec<char>>();
//     let r = parse_section_title(&input);
//     assert_eq!(["section".to_string(), "pretty.hard".to_string(), "nested".to_string()],
//                r.as_slice());
// }

// #[test]
// fn parse_inner_harder() {
//     let input = "section.\"quote \"some\" words\".nested".chars().collect::<Vec<char>>();
//     let r = parse_section_title(&input);
//     assert_eq!(["section".to_string(), "quote \"some\" words".to_string(), "nested".to_string()],
//                r.as_slice());
// }

// #[test]
// fn section_title_to_table_name() {
//     let mut idx = 0;
//     let input = r#"[section."pretty.hard".nested]"#.to_string().chars().collect::<Vec<char>>();
//     let r = section_title_to_subsections(&input, &mut idx);
//     assert_eq!([String::from("section"), String::from("pretty.hard"), String::from("nested")],
//                r.as_slice());
// }

// #[test]
// fn key_bare() {
//     let input = "bare_key = 15".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("bare_key".to_string()),
//         value: Value::Integer(15),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn key_quoted() {
//     // TODO: Escaped quotes in quoted strings
//     let input = "\"Fancy Quoted K3y\" = 15".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Quoted("Fancy Quoted K3y".to_string()),
//         value: Value::Integer(15),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_string() {
//     // Regular spacing
//     let input = "keyname = \"valname\"".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::SString("valname".to_string()),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     // Crazy spacing
//     idx = 0;
//     let input = "keyname  \t   =  \"valname\"   ".chars().collect::<Vec<char>>();
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_int() {
//     let input = "keyname = 15".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Integer(15),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     let input = "keyname = 150_263  ".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Integer(150263),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     let input = "keyname = -150_263 ".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Integer(-150263),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_float() {
//     let input = "keyname = 15.5".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Float(15.5),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     let input = "keyname = -0.01".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Float(-0.01),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     let input = "keyname = -5e+22".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Float(-5e+22),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_bool() {
//     let input = "keyname = true".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Bool(true),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     let input = "keyname = false".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Bool(false),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_datetime() {
//     let input = "SomeDate = 1979-05-27T00:32:00.999999-07:00".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     // Not the world's most useful comparison
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("SomeDate".to_string()),
//         value: Value::DateTime(ChronoDateTime::parse_from_rfc3339("1979-05-27T00:32:00.\
//                                                                    999999-07:00")
//             .unwrap()),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_array_int() {
//     let input = "my_array = [1, 2, 3]".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     // Not the world's most useful comparison
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("my_array".to_string()),
//         value: Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_array_bool() {
//     let input = "my_array = [true, false, true]".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("my_array".to_string()),
//         value: Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)]),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_array_string() {
//     let input = r#"my_array = ["test", "test", "test"]"#.chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("my_array".to_string()),
//         value: Value::Array(vec![Value::SString("test".to_string()),
//                                  Value::SString("test".to_string()),
//                                  Value::SString("test".to_string())]),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_array_array() {
//     let input = r#"my_array = [[1, 2, 3], [1, 2, 3], [1, 2, 3]]"#.chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("my_array".to_string()),
//         value: Value::Array(vec![Value::Array(vec![Value::Integer(1),
//                                                    Value::Integer(2),
//                                                    Value::Integer(3)]),
//                                  Value::Array(vec![Value::Integer(1),
//                                                    Value::Integer(2),
//                                                    Value::Integer(3)]),
//                                  Value::Array(vec![Value::Integer(1),
//                                                    Value::Integer(2),
//                                                    Value::Integer(3)])]),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_with_comment() {
//     let input = "keyname = 15 # This is a comment".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Integer(15),
//         comment: Some(Comment {
//             indent: " ".to_string(),
//             comment: " This is a comment".to_string(),
//         }),
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_with_comment_no_space() {
//     let input = "keyname = 15#This is a comment".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("keyname".to_string()),
//         value: Value::Integer(15),
//         comment: Some(Comment {
//             indent: "".to_string(),
//             comment: "This is a comment".to_string(),
//         }),
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn indent_keyval() {
//     let input = "\t\tindent = \"Two tabs\"".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "\t\t".to_string(),
//         key: Key::Bare("indent".to_string()),
//         value: Value::SString("Two tabs".to_string()),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));

//     let input = "      indent = \"Two tabs\"".chars().collect::<Vec<char>>();
//     let mut idx = 0;
//     let correct = KeyValue {
//         indent: "      ".to_string(),
//         key: Key::Bare("indent".to_string()),
//         value: Value::SString("Two tabs".to_string()),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_inline_table() {
//     let input = "inline = {a = 1, b = 2, c = 3}".chars().collect::<Vec<char>>();
//     let mut idx = 0;

//     let a = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("a".to_string()),
//         value: Value::Integer(1),
//         comment: None,
//     };

//     let b = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("b".to_string()),
//         value: Value::Integer(2),
//         comment: None,
//     };

//     let c = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("c".to_string()),
//         value: Value::Integer(3),
//         comment: None,
//     };
//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("inline".to_string()),
//         value: Value::InlineTable(vec![a, b, c]),
//         comment: None,
//     };
//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// #[test]
// fn keyval_inline_table_string() {
//     let input = "inline = {a = \"a\", b=\"b\", c = \"c\"}".chars().collect::<Vec<char>>();
//     let mut idx = 0;

//     let a = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("a".to_string()),
//         value: Value::SString("a".to_string()),
//         comment: None,
//     };

//     let b = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("b".to_string()),
//         value: Value::SString("b".to_string()),
//         comment: None,
//     };

//     let c = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("c".to_string()),
//         value: Value::SString("c".to_string()),
//         comment: None,
//     };

//     let correct = KeyValue {
//         indent: "".to_string(),
//         key: Key::Bare("inline".to_string()),
//         value: Value::InlineTable(vec![a, b, c]),
//         comment: None,
//     };

//     assert_eq!(correct, parse_key_value(&input, &mut idx));
// }

// TODO: keyvalue factory
// Keyval test structure: 1 simple standalone test + 1 fn with edge cases
