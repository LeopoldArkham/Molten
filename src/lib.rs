#![allow(dead_code, non_snake_case)]

// extern crate linked_hash_map;

// use linked_hash_map::LinkedHashMap;

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct TableName(String);

#[derive(Debug, PartialEq)]
pub enum Value {
    SString(String), // Quote
    Integer(i64), // Digit
    Float(f64), // Digit
    Bool(bool), // char
    Datetime, // char
    Array, // Bracket
    InlineTable, // Curly bracket
}


impl Value {
    // Todo: rename
    fn from_str(input: &[char]) -> Value {
        use Value::*;
        match input[0] {
            '"' => {
                // TODO: Clever iterator trick with count()?
                let mut idx = 1;
                while input[idx] != '"' {
                    idx += 1;
                }
                SString(input[1..idx].iter().map(|c| *c).collect::<String>())
            }
            't' => Bool(true),
            'f' => Bool(false),
            // TODO
            '[' => Array,
            // TODO: Subparser inherits main parser
            '{' => InlineTable,
            // TODO: Try parse int => float => datetime
            '+' | '-' | '0'...'9' => {
                let mut idx = 0;
                // TODO: Really need capped integers...
                while idx != input.len() - 1 && input[idx + 1].is_int_float_char() {
                    idx += 1;
                }

                let clean =
                    input[..idx + 1].iter().filter(|c| **c != '_').map(|c| *c).collect::<String>();

                if let Ok(res) = i64::from_str(&clean) {
                    return Integer(res);
                } else if let Ok(res) = f64::from_str(&clean) {
                    return Float(res);
                }
                // TODO: Datetime here
                panic!("Could not parse to int or float");
            }
            _ => Datetime,
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


// TODO: Remove unused
impl TableName {
    pub fn new<T: Into<String>>(name: T) -> TableName {
        TableName(name.into())
    }
}

// TODO: Dispatch on value content and parse to appropriate value type
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
    fn is_bare_key_char(&self) -> bool;
    fn is_ws_or_equal(&self) -> bool;
    fn is_int_float_char(&self) -> bool;
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

    let val_start = idx;
    let val = Value::from_str(&input[val_start..]);

    KeyValue {
        key: key,
        value: val,
        comment: None,
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

pub fn section_title_to_subsections(input: &[char]) -> Vec<TableName> {
    let inner = remove_brackets(input);
    let names = parse_section_title(inner);
    names.into_iter().map(TableName).collect()
}

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
    assert_eq!([TableName::new("section"),
                TableName::new("pretty.hard"),
                TableName::new("nested")],
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