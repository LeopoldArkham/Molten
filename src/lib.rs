#![allow(dead_code, non_snake_case)]

extern crate linked_hash_map;

use linked_hash_map::LinkedHashMap;

#[derive(Debug, PartialEq)]
pub struct TableName(String);

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String), // Quote
    Integer, // Digit
    Float, // Digit
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
            '"' => String("s".to_string()),
            't' => Bool(true),
            'f' => Bool(false),
            // TODO: Use nom for array parsing?
            '[' => Array,
            // TODO: Subparser inherits main parser
            '{' => InlineTable,
            // TODO: Try parse int => float => datetime
            '0'...'9' => Integer,
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
pub struct KeyValue {
    key: String,
    value: Value,
    comment: String,
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
}

impl TOMLChar for char {
    fn is_bare_key_char(&self) -> bool {
        match *self {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => true,
            _ => false,
        }
    }
}

pub fn parse_key_value(input: &[char]) -> KeyValue {
    let mut idx = 0;

    while input[idx].is_bare_key_char() {
        idx += 1;
    }

    let key = input[0..idx].iter().map(|c| *c).collect::<String>();

    while !input[idx].is_bare_key_char() {
        idx += 1;
    }

    let val_start = idx;

    // TODO: idx capped integer?
    while input[idx].is_bare_key_char() && idx != input.len() - 1 {
        idx += 1;
    }

    // This is so the last value char is not truncated when it is the last char of the input.
    let val_end = if idx == input.len() - 1 {
        input.len()
    } else {
        idx
    };

    let val = input[val_start..val_end].iter().map(|c| *c).collect::<String>();

    KeyValue {
        key: key,
        value: Value::String(val),
        comment: "".to_string(),
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
fn name() {
    let input = r#"\""#;
    println!("{:?}", input.chars().collect::<Vec<_>>());
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
    // println!("{:?}", input);
    let r = section_title_to_subsections(&input);
    assert_eq!([TableName::new("section"),
                TableName::new("pretty.hard"),
                TableName::new("nested")],
               r.as_slice());
}

#[test]
fn keyval_easy() {
    // Regular spacing
    let input = "keyname = valname".chars().collect::<Vec<char>>();
    let correct = KeyValue {
        key: "keyname".to_string(),
        value: Value::String("valname".to_string()),
        comment: "".to_string(),
    };
    assert_eq!(correct, parse_key_value(&input));

    // Crazy spacing
    let input = "keyname   =  valname  ".chars().collect::<Vec<char>>();
    assert_eq!(correct, parse_key_value(&input));
}