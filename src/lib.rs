#![allow(dead_code, non_snake_case)]

// extern crate linked_hash_map;
extern crate chrono;

// use linked_hash_map::LinkedHashMap;

use std::str::FromStr;
use chrono::{DateTime as ChronoDateTime, FixedOffset};

#[derive(Debug, PartialEq)]
pub enum Value {
    SString(String), // Quote
    Integer(i64), // Digit, +, -
    Float(f64), // Digit, +, -
    Bool(bool), // char
    DateTime(ChronoDateTime<FixedOffset>), // Digit
    Array, // Bracket
    InlineTable, // Curly bracket
}

// Next up:
// Awareness of indentation and ws before comments in keyval parsing
// Keyval to str Display implementation

impl Value {
    // Todo: rename
    fn from_str(input: &[char], idx: &mut usize) -> Value {
        use Value::*;
        // println!("{:#?}", &input[*idx..]);
        let start_idx = *idx;
        match input[*idx] {
            '"' => {
                // TODO: Clever iterator trick with count()?
                *idx += 1;
                while input[*idx] != '"' {
                    *idx += 1;
                }
                SString(input[start_idx + 1..*idx].iter().map(|c| *c).collect::<String>())
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
            '[' => Array,
            // TODO: Subparser inherits main parser
            '{' => InlineTable,
            // TODO: Try parse int => float => datetime
            '+' | '-' | '0'...'9' => {
                // TODO: Really need capped integers...
                // TODO: '#' char could be appended with no space
                while *idx != input.len() - 1 && input[*idx + 1].not_whitespace_or_pound() {
                    *idx += 1;
                }

                // TODO: Filtermap and why **?
                let clean = input[start_idx..*idx + 1]
                    .iter()
                    .filter(|c| **c != '_')
                    .map(|c| *c)
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

#[derive(Debug, PartialEq)]
pub struct KeyValue {
    key: Key,
    value: Value,
    comment: Option<String>,
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


pub trait TOMLChar {
    // TODO Add validators for each key/value scheme
    // Remove all shameful things
    fn is_bare_key_char(&self) -> bool;
    fn is_ws_or_equal(&self) -> bool;
    fn is_int_float_char(&self) -> bool;
    fn not_whitespace(&self) -> bool;
    fn not_whitespace_or_pound(&self) -> bool;
}

impl TOMLChar for char {
    fn is_bare_key_char(&self) -> bool {
        match *self {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => true,
            _ => false,
        }
    }

    fn is_ws_or_equal(&self) -> bool {
        match *self {
            '=' | ' ' | '\t' => true,
            _ => false,
        }
    }

    fn is_int_float_char(&self) -> bool {
        match *self {
            '+' | '-' | '_' | '0'...'9' | 'e' | '.' => true,
            _ => false,
        }
    }

    fn not_whitespace(&self) -> bool {
        match *self {
            ' ' | '\t' => false,
            _ => true,
        }
    }

    fn not_whitespace_or_pound(&self) -> bool {
        match *self {
            ' ' | '\t' | '#' => false,
            _ => true,
        }
    }
}

pub fn parse_quoted_key(input: &[char], idx: &mut usize) -> Key {
    // Skip "
    *idx += 1;
    while input[*idx] != '"' {
        *idx += 1;
    }
    let key = input[1..*idx].iter().map(|c| *c).collect::<String>();
    // Skip "
    *idx += 1;

    Key::Quoted(key)
}

pub fn parse_bare_key(input: &[char], idx: &mut usize) -> Key {
    while input[*idx].is_bare_key_char() {
        *idx += 1;
    }

    let key = input[0..*idx].iter().map(|c| *c).collect::<String>();

    Key::Bare(key)

}

fn parse_comment(input: &[char], idx: &mut usize) -> Option<String> {
    loop {
        if input[*idx] == '#' {
            return Some(input[*idx + 1..].iter().map(|c| *c).collect::<String>());
        }
        if *idx == input.len() - 1 {
            return None;
        }
        *idx += 1;
    }
}

pub fn parse_key_value(input: &[char]) -> KeyValue {
    let mut idx = 0;

    let key = match input[0] {
        '"' => parse_quoted_key(&input, &mut idx),
        _ => parse_bare_key(&input, &mut idx),
    };

    // Skip = and whitespace
    while input[idx].is_ws_or_equal() {
        idx += 1;
    }

    let val = Value::from_str(&input, &mut idx);

    // Assume whitespace before #
    let comment = parse_comment(&input, &mut idx);

    KeyValue {
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
            chr @ _ => current_name.push(chr),
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
    let correct = KeyValue {
        key: Key::Bare("bare_key".to_string()),
        value: Value::Integer(15),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn key_quoted() {
    // TODO: Escaped quotes in quoted strings
    let input = "\"Fancy Quoted K3y\" = 15".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Quoted("Fancy Quoted K3y".to_string()),
        value: Value::Integer(15),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_string() {
    // Regular spacing
    let input = "keyname = \"valname\"".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::SString("valname".to_string()),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));

    // Crazy spacing
    let input = "keyname \t  =  \"valname\"  ".chars().collect::<Vec<char>>();
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_int() {
    let input = "keyname = 15".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(15),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));

    let input = "keyname = 150_263  ".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(150263),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));

    let input = "keyname = -150_263 ".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(-150263),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_float() {
    let input = "keyname = 15.5".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Float(15.5),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));

    let input = "keyname = -0.01".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Float(-0.01),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));

    let input = "keyname = -5e+22".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Float(-5e+22),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_bool() {
    let input = "keyname = true".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Bool(true),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));

    let input = "keyname = false".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Bool(false),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_datetime() {
    let input = "SomeDate = 1979-05-27T00:32:00.999999-07:00".chars().collect::<Vec<char>>();
    // Not the world's most useful comparison
    let correct = KeyValue {
        key: Key::Bare("SomeDate".to_string()),
        value: Value::DateTime(ChronoDateTime::parse_from_rfc3339("1979-05-27T00:32:00.\
                                                                   999999-07:00")
            .unwrap()),
        comment: None,
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_with_comment() {
    let input = "keyname = 15 # This is a comment".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(15),
        comment: Some(String::from(" This is a comment")),
    };
    assert_eq!(correct, parse_key_value(&input));
}

#[test]
fn keyval_with_comment_no_space() {
    let input = "keyname = 15#This is a comment".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: Key::Bare("keyname".to_string()),
        value: Value::Integer(15),
        comment: Some(String::from("This is a comment")),
    };
    assert_eq!(correct, parse_key_value(&input));
}

// TODO: keyvalue factory
// Keyval test structure: 1 simple standalone test + 1 fn with edge cases