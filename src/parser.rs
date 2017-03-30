use tomldoc::TOMLDocument;
use tomlchar::TOMLChar;
use Comment;
use value::Value;
use Table;
use KeyValue;
use Key;

use chrono::{DateTime as ChronoDateTime, FixedOffset};

use std::str::FromStr;

#[derive(Debug)]
pub enum TLV {
    WS(String),
    Val(KeyValue),
}

#[derive(Debug)]
pub struct Parser {
    src: Vec<char>,
    idx: usize,
    marker: usize,
}

impl Parser {
    /// Create a new parser from a string.
    pub fn new(input: &str) -> Parser {
        Parser {
            src: input.chars().collect::<Vec<char>>(),
            idx: 0,
            marker: 0,
        }
    }

    /// Moves the marker to the index's current position
    fn mark(&mut self) {
        self.marker = self.idx;
    }

    /// Parses the input into a TOMLDocument
    pub fn parse(&mut self) -> TOMLDocument {
        let mut body = Vec::new();
        
        let leading_ws = self.take_ws();
        body.push(leading_ws);
        // TODO: Toplevel enum
        let nextval = self.parse_key_value();
        let wrapper = TLV::Val(nextval);
        body.push(wrapper);

        TOMLDocument(body)
    }

    /// Advances the parser to the first non-whitespce character
    /// and returns the consumed whitespace as a string.
    pub fn take_ws(&mut self) -> TLV {
        self.mark();
        while self.src[self.idx].is_ws() {
            self.idx += 1;
        }
        
        TLV::WS(self.src[self.marker..self.idx].iter().collect::<String>())
    }

    /// Parses and returns a key/value pair.
    pub fn parse_key_value(&mut self) -> KeyValue {
        self.mark();
        while self.src[self.idx].is_whitespace() {
            self.idx += 1;
        }

        let indent = self.src[self.marker..self.idx].iter().cloned().collect::<String>();

        let key = match self.src[self.idx] {
            '"' => self.parse_quoted_key(),
            _ => self.parse_bare_key(),
        };

        // Skip = and whitespace
        while self.src[self.idx].is_ws_or_equal() {
            self.idx += 1;
        }

        let val = self.parse_val();

        let comment = if self.idx == self.src.len() - 1 {
            None
        } else {
            self.idx += 1;
            self.parse_comment()
        };

        KeyValue {
            indent: indent,
            key: key,
            value: val,
            comment: comment,
        }
    }

    /// Attempts to parse a value at the current position.
    pub fn parse_val(&mut self) -> Value {
        use Value::*;
        self.mark();
        match self.src[self.idx] {
            '"' => {
                // TODO: Clever iterator trick with count()?
                self.idx += 1;
                while self.src[self.idx] != '"' {
                    self.idx += 1;
                    if self.idx == self.src.len() {
                        println!("{:?}", &self.src[self.marker..]);
                    }
                }
                SString(self.src[self.marker + 1..self.idx].iter().cloned().collect::<String>())
            }
            't' if self.src[self.idx..self.idx + 4] == ['t', 'r', 'u', 'e'] => {
                self.idx += 3;
                Bool(true)
            }
            'f' if self.src[self.idx..self.idx + 5] == ['f', 'a', 'l', 's', 'e'] => {
                self.idx += 4;
                Bool(false)
            }
            '[' => {
                let mut elems: Vec<Value> = Vec::new();
                self.idx += 1;

                while self.src[self.idx] != ']' {
                    while self.src[self.idx].is_ws() || self.src[self.idx] == ',' {
                        self.idx += 1;
                    }
                    let val = self.parse_val();
                    self.idx += 1;
                    let check = val.discriminant();
                    elems.push(val);
                    assert_eq!(elems[0].discriminant(), check);
                }
                Array(elems)
            }
            '{' => {
                let mut elems: Vec<KeyValue> = Vec::new();
                self.idx += 1;

                while self.src[self.idx] != '}' {
                    while self.src[self.idx].is_ws() || self.src[self.idx] == ',' {
                        self.idx += 1;
                    }
                    let val = self.parse_key_value();
                    elems.push(val);
                }

                InlineTable(elems)
            }
            // TODO: Try parse int => float => datetime
            '+' | '-' | '0'...'9' => {
                // TODO: Really need capped integers...
                // TODO: '#' char could be appended with no space

                // Send help.
                while self.idx != self.src.len() - 1 && self.src[self.idx + 1].not_whitespace_or_pound() &&
                      self.src[self.idx + 1] != ',' && self.src[self.idx + 1] != ']' &&
                      self.src[self.idx + 1] != '}' {
                    self.idx += 1;
                }

                // TODO: Filtermap and why **?
                let clean = self.src[self.marker..self.idx + 1]
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

                println!("working on: {:?}", clean);
                panic!("Could not parse to int, float or DateTime");
            }
            _ => panic!("Could not infer type of value being parsed"),
        }
    }

    fn parse_comment(&mut self) -> Option<Comment> {
        self.mark();
        loop {
            if self.src[self.idx] == '#' {
                // TODO: Simplify
                let indent = if self.marker == self.idx {
                    "".to_string()
                } else {
                    self.src[self.marker..self.idx].iter().cloned().collect::<String>()
                };

                let comment = self.src[self.idx + 1..].iter().cloned().collect::<String>();
                return Some(Comment {
                    indent: indent,
                    comment: comment,
                });
            }
            if self.idx == self.src.len() - 1 {
                self.idx = self.marker;
                return None;
            }
            self.idx += 1;
        }
    }

    pub fn remove_brackets<'a>(&mut self) -> String {
        // TODO: Allow brackets in quoted names
        while self.src[self.idx] != '[' {
            self.idx += 1;
        }

        self.idx += 1;
        self.mark();

        while self.src[self.idx] != ']' {
            self.idx += 1;
        }

        self.src[self.marker..self.idx].iter().cloned().collect::<String>()
    }

    pub fn parse_quoted_key(&mut self) -> Key {
        // Skip "
        self.idx += 1;
        self.mark();

        while self.src[self.idx] != '"' {
            self.idx += 1;
        }

        let key = self.src[self.marker..self.idx].iter().cloned().collect::<String>();
        // Skip "
        self.idx += 1;

        Key::Quoted(key)
    }

    pub fn parse_bare_key(&mut self) -> Key {
        self.mark();
        while self.src[self.idx].is_bare_key_char() {
            self.idx += 1;
        }
        let key = self.src[self.marker..self.idx].iter().cloned().collect::<String>();
        Key::Bare(key)
    }



    pub fn parse_section_title(&mut self) -> Vec<String> {
        // TODO: Escaped quotes in quoted table names
        let mut in_quotes = false;
        let mut names = Vec::new();
        let mut current_name = String::new();

        for (i, c) in self.src.iter().enumerate() {
            match *c {
                '"' if self.src[self.idx + 1] == '.' => in_quotes = !in_quotes,
                '"' if self.src[self.idx - 1] == '.' => in_quotes = !in_quotes,
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

    // TODO: Better names everywhere
    pub fn section_title_to_subsections(&mut self) -> Vec<String> {
        let inner = self.remove_brackets();
        let names = self.parse_section_title();
        names.into_iter().map(String::from).collect()
    }

    pub fn parse_table(&mut self) -> Table {
        let title = self.section_title_to_subsections();
        let mut values = Vec::new();

        while self.idx < self.src.len() - 1 {
            while self.idx != self.src.len() - 1 && self.src[self.idx] != '\n' {
                self.idx += 1;
            }
            self.idx += 1;

            let val = self.parse_key_value();
            values.push(val)
        }

        // TODO: name || Title
        Table {
            name: title,
            comment: "".to_string(),
            values: values,
        }

    }
}