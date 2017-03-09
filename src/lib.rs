#![allow(dead_code, non_snake_case)]

// extern crate linked_hash_map;
extern crate chrono;

mod tomlchar;

use std::str::FromStr;

// use linked_hash_map::LinkedHashMap;
use chrono::{DateTime as ChronoDateTime, FixedOffset};

use tomlchar::TOMLChar;

#[derive(Debug, PartialEq)]
pub enum Value {
    SString(String), // Quote
    Integer(i64), // Digit, +, -
    Float(f64), // Digit, +, -
    Bool(bool), // char
    DateTime(ChronoDateTime<FixedOffset>), // Digit
    Array(Vec<Value>), // Bracket
    InlineTable(Vec<KeyValue>), // Curly bracket
}

// Next up:
// Switch keyval repr to ast-like structure for full ws awareness
// Keyval to str Display implementation -- After ^
// Delimiter matching engine. Until then, assume unambiguous delmiters.

impl Value {
    // Todo: rename
    fn from_str(input: &[char], idx: &mut usize) -> Value {
        use Value::*;
        let start_idx = *idx;
        match input[*idx] {
            '"' => {
                // TODO: Clever iterator trick with count()?
                *idx += 1;
                while input[*idx] != '"' {
                    *idx += 1;
                    if *idx == input.len() {
                        println!("{:?}", &input[start_idx..]);
                    }
                }
                SString(input[start_idx + 1..*idx].iter().cloned().collect::<String>())
            }
            't' if input[*idx..*idx + 4] == ['t', 'r', 'u', 'e'] => {
                *idx += 3;
                Bool(true)
            }
            'f' if input[*idx..*idx + 5] == ['f', 'a', 'l', 's', 'e'] => {
                *idx += 4;
                Bool(false)
            }
            // TODO
            '[' => {
                // Stagger ws /\ value
                // Map parsing to values
                // Assert homogeneity
                // Zip() ws /\ parsed values

                // Subtype: Value | WS Enum | Comment iff EOL
                let mut elems: Vec<Value> = Vec::new();
                *idx += 1;

                while input[*idx] != ']' {
                    while input[*idx].is_ws() || input[*idx] == ',' {
                        *idx += 1;
                    }
                    let val = Value::from_str(input, idx);
                    *idx += 1;
                    let check = val.as_usize();
                    elems.push(val);
                    assert_eq!(elems[0].as_usize(), check);
                }
                Array(elems)
            }
            // TODO: Subparser inherits main parser
            '{' => {
                let mut elems: Vec<KeyValue> = Vec::new();
                *idx += 1;

                while input[*idx] != '}' {
                    while input[*idx].is_ws() || input[*idx] == ',' {
                        *idx += 1;
                    }
                    let val = parse_key_value(input, idx);
                    elems.push(val);
                }

                InlineTable(elems)
            }
            // TODO: Try parse int => float => datetime
            '+' | '-' | '0'...'9' => {
                // TODO: Really need capped integers...
                // TODO: '#' char could be appended with no space

                // Send help.
                while *idx != input.len() - 1 && input[*idx + 1].not_whitespace_or_pound() &&
                      input[*idx + 1] != ',' && input[*idx + 1] != ']' && input[*idx + 1] != '}' {
                    *idx += 1;
                }

                // TODO: Filtermap and why **?
                let clean = input[start_idx..*idx + 1]
                    .iter()
                    .filter(|c| **c != '_')
                    .cloned()
                    .collect::<String>();

                // Ask forgiveness, not permission
                if let Ok(res) = i64::from_str(&clean) {
                    return Integer(res);
                } else if let Ok(res) = f64::from_str(&clean) {
                    return Float(res);
                } else if let Ok(res) = ChronoDateTime::parse_from_rfc3339(&clean) {
                    return DateTime(res);
                }

                panic!("Could not parse to int, float or DateTime");
            }
            _ => panic!("Could not infer type of value being parsed"),
        }
    }

    fn as_usize(&self) -> usize {
        match *self {
            Value::SString(_) => 1  as usize,
            Value::Integer(_) => 2 as usize,
            Value::Float(_) => 3 as usize,
            Value::Bool(_) => 4 as usize,
            Value::DateTime(_) => 5 as usize,
            Value::Array(_) => 6 as usize,
            Value::InlineTable(_) => 7 as usize,

        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Table {
    name: String,
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

// TODO: Stateful parser
// TODO: Result type


pub fn remove_brackets(input: &[char]) -> &[char] {
    // TODO: Allow brackets in quoted names
    // TODO: Use nom here
    let mut idx = 0;

    while input[idx] != '[' {
        idx += 1;
    }

    idx += 1;
    let start = idx;

    while input[idx] != ']' {
        idx += 1;
    }

    &input[start..idx]
}

pub fn parse_quoted_key(input: &[char], idx: &mut usize) -> Key {
    // Skip "
    *idx += 1;
    let start_idx = *idx;

    while input[*idx] != '"' {
        *idx += 1;
    }

    let key = input[start_idx..*idx].iter().cloned().collect::<String>();
    // Skip "
    *idx += 1;

    Key::Quoted(key)
}

pub fn parse_bare_key(input: &[char], idx: &mut usize) -> Key {
    let start_idx = *idx;

    while input[*idx].is_bare_key_char() {
        *idx += 1;
    }

    let key = input[start_idx..*idx].iter().cloned().collect::<String>();

    Key::Bare(key)

}

fn parse_comment(input: &[char], idx: &mut usize) -> Option<Comment> {
    let start_idx = *idx;
    loop {
        if input[*idx] == '#' {
            // TODO: Simplify
            let indent = if start_idx == *idx {
                "".to_string()
            } else {
                input[start_idx..*idx].iter().cloned().collect::<String>()
            };

            let comment = input[*idx + 1..].iter().cloned().collect::<String>();
            return Some(Comment {
                indent: indent,
                comment: comment,
            });
        }
        if *idx == input.len() - 1 {
            *idx = start_idx; 
            return None;
        }
        *idx += 1;
    }
}

pub fn parse_key_value(input: &[char], idx: &mut usize) -> KeyValue {
    let start_idx = *idx;
    while input[*idx].is_whitespace() {
        *idx += 1;
    }

    let indent = input[start_idx..*idx].iter().cloned().collect::<String>();

    let key = match input[*idx] {
        '"' => parse_quoted_key(input, idx),
        _ => parse_bare_key(input, idx),
    };

    // Skip = and whitespace
    while input[*idx].is_ws_or_equal() {
        *idx += 1;
    }

    let val = Value::from_str(input, idx);

    let comment = if *idx == input.len() - 1 {
        None
    } else {
        *idx += 1;
        parse_comment(input, idx)
    };

    KeyValue {
        indent: indent,
        key: key,
        value: val,
        comment: comment,
    }

}

pub fn parse_section_title(input: &[char]) -> Vec<String> {
    // TODO: Escaped quotes in quoted table names
    let mut in_quotes = false;
    let mut names = Vec::new();
    let mut current_name = String::new();

    for (i, c) in input.iter().enumerate() {
        match *c {
            '"' if input[i + 1] == '.' => in_quotes = !in_quotes,
            '"' if input[i - 1] == '.' => in_quotes = !in_quotes,
            '.' if !in_quotes => {
                names.push(current_name.clone());
                current_name.drain(..);
            }
            chr => current_name.push(chr),
        }
    }
    names.push(current_name.clone());
    names
}

pub fn section_title_to_subsections(input: &[char]) -> Vec<String> {
    let inner = remove_brackets(input);
    let names = parse_section_title(inner);
    names.into_iter().map(String::from).collect()
}

// TESTS

#[test]
fn t_remove_brackets() {
    let input = "[Section]".to_string().chars().collect::<Vec<char>>();
    let r = remove_brackets(&input).iter().map(|c| *c).collect::<String>();
    assert_eq!("Section", &r);

    let input = "[section.nested]".to_string().chars().collect::<Vec<char>>();
    let r = remove_brackets(&input).iter().map(|c| *c).collect::<String>();
    assert_eq!("section.nested", &r);
}

#[test]
fn parse_inner_easy() {
    let input = "section.nested".to_string().chars().collect::<Vec<char>>();
    let r = parse_section_title(&input);
    assert_eq!(["section".to_string(), "nested".to_string()], r.as_slice());
}

#[test]
fn parse_inner_hard() {
    let input = "section.\"pretty.hard\".nested".to_string().chars().collect::<Vec<char>>();
    let r = parse_section_title(&input);
    assert_eq!(["section".to_string(), "pretty.hard".to_string(), "nested".to_string()],
               r.as_slice());
}

#[test]
fn parse_inner_harder() {
    let input = "section.\"quote \"some\" words\".nested".chars().collect::<Vec<char>>();
    let r = parse_section_title(&input);
    assert_eq!(["section".to_string(), "quote \"some\" words".to_string(), "nested".to_string()],
               r.as_slice());
}

#[test]
fn section_title_to_table_name() {
    let input = r#"[section."pretty.hard".nested]"#.to_string().chars().collect::<Vec<char>>();
    let r = section_title_to_subsections(&input);
    assert_eq!([String::from("section"), String::from("pretty.hard"), String::from("nested")],
               r.as_slice());
}

#[test]
fn key_bare() {
    let input = "bare_key = 15".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("bare_key".to_string()),
        value: Value::Integer(15),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn key_quoted() {
    // TODO: Escaped quotes in quoted strings
    let input = "\"Fancy Quoted K3y\" = 15".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Quoted("Fancy Quoted K3y".to_string()),
        value: Value::Integer(15),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_string() {
    // Regular spacing
    let input = "keyname = \"valname\"".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::SString("valname".to_string()),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    // Crazy spacing
    idx = 0;
    let input = "keyname  \t   =  \"valname\"   ".chars().collect::<Vec<char>>();
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_int() {
    let input = "keyname = 15".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(15),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    let input = "keyname = 150_263  ".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(150263),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    let input = "keyname = -150_263 ".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(-150263),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_float() {
    let input = "keyname = 15.5".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Float(15.5),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    let input = "keyname = -0.01".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Float(-0.01),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    let input = "keyname = -5e+22".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Float(-5e+22),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_bool() {
    let input = "keyname = true".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Bool(true),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    let input = "keyname = false".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Bool(false),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_datetime() {
    let input = "SomeDate = 1979-05-27T00:32:00.999999-07:00".chars().collect::<Vec<char>>();
    let mut idx = 0;
    // Not the world's most useful comparison
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("SomeDate".to_string()),
        value: Value::DateTime(ChronoDateTime::parse_from_rfc3339("1979-05-27T00:32:00.\
                                                                   999999-07:00")
            .unwrap()),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_array_int() {
    let input = "my_array = [1, 2, 3]".chars().collect::<Vec<char>>();
    let mut idx = 0;
    // Not the world's most useful comparison
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("my_array".to_string()),
        value: Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_array_bool() {
    let input = "my_array = [true, false, true]".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("my_array".to_string()),
        value: Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)]),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_array_string() {
    let input = r#"my_array = ["test", "test", "test"]"#.chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("my_array".to_string()),
        value: Value::Array(vec![Value::SString("test".to_string()),
                                 Value::SString("test".to_string()),
                                 Value::SString("test".to_string())]),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_array_array() {
    let input = r#"my_array = [[1, 2, 3], [1, 2, 3], [1, 2, 3]]"#.chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("my_array".to_string()),
        value: Value::Array(vec![Value::Array(vec![Value::Integer(1),
                                                   Value::Integer(2),
                                                   Value::Integer(3)]),
                                 Value::Array(vec![Value::Integer(1),
                                                   Value::Integer(2),
                                                   Value::Integer(3)]),
                                 Value::Array(vec![Value::Integer(1),
                                                   Value::Integer(2),
                                                   Value::Integer(3)])]),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_with_comment() {
    let input = "keyname = 15 # This is a comment".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(15),
        comment: Some(Comment {
            indent: " ".to_string(),
            comment: " This is a comment".to_string(),
        }),
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_with_comment_no_space() {
    let input = "keyname = 15#This is a comment".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(15),
        comment: Some(Comment {
            indent: "".to_string(),
            comment: "This is a comment".to_string(),
        }),
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn indent_keyval() {
    let input = "\t\tindent = \"Two tabs\"".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "\t\t".to_string(),
        key: Key::Bare("indent".to_string()),
        value: Value::SString("Two tabs".to_string()),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));

    let input = "      indent = \"Two tabs\"".chars().collect::<Vec<char>>();
    let mut idx = 0;
    let correct = KeyValue {
        indent: "      ".to_string(),
        key: Key::Bare("indent".to_string()),
        value: Value::SString("Two tabs".to_string()),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_inline_table() {
    let input = "inline = {a = 1, b = 2, c = 3}".chars().collect::<Vec<char>>();
    let mut idx = 0;

    let a = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("a".to_string()),
        value: Value::Integer(1),
        comment: None
    };

    let b = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("b".to_string()),
        value: Value::Integer(2),
        comment: None
    };

    let c = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("c".to_string()),
        value: Value::Integer(3),
        comment: None
    };
    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("inline".to_string()),
        value: Value::InlineTable(vec![a, b, c]),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

#[test]
fn keyval_inline_table_string() {
    let input = "inline = {a = \"a\", b=\"b\", c = \"c\"}".chars().collect::<Vec<char>>();
    let mut idx = 0;

    let a = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("a".to_string()),
        value: Value::SString("a".to_string()),
        comment: None
    };

    let b = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("b".to_string()),
        value: Value::SString("b".to_string()),
        comment: None
    };

    let c = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("c".to_string()),
        value: Value::SString("c".to_string()),
        comment: None
    };

    let correct = KeyValue {
        indent: "".to_string(),
        key: Key::Bare("inline".to_string()),
        value: Value::InlineTable(vec![a, b, c]),
        comment: None,
    };

    assert_eq!(correct, parse_key_value(&input, &mut idx));
}

// TODO: keyvalue factory
// Keyval test structure: 1 simple standalone test + 1 fn with edge cases
