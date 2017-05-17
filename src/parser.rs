use tomldoc::TOMLDocument;
use tomlchar::TOMLChar;
use TOMLElements::*;

use chrono::{DateTime as ChronoDateTime, FixedOffset};

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum TLV {
    WS(String),
    Comment(Comment),
    Val(KeyValue),
    Table(Table),
    AoT(Vec<TLV>),
}

impl TLV {
    pub fn as_string(&self) -> String {
        match *self {
            TLV::WS(ref s) => s.clone(),
            TLV::Comment(ref c) => c.as_string(),
            TLV::Val(ref kv) => kv.as_string(),
            TLV::Table(ref t) => t.as_string(),
            TLV::AoT(ref vec) => {
                let mut res = String::new();
                for e in vec.iter() {
                    res.push_str(&e.as_string());
                }
                res
            },
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    src: Vec<char>,
    idx: usize,
    marker: usize,
    end: usize,
}

impl Parser {
    /// Create a new parser from a string.
    pub fn new(input: &str) -> Parser {
        Parser {
            src: input.chars().collect::<Vec<char>>(),
            idx: 0,
            marker: 0,
            end: input.len() - 1 as usize,
        }
    }

    /// Extract the value between marker and index.
    fn extract(&self) -> String {
        self.src[self.marker..self.idx].iter().cloned().collect::<String>()
    }

    ///Sets the marker to the index's current position
    fn mark(&mut self) {
        self.marker = self.idx;
    }

    /// Sets the marker to the specified position
    fn mark_at(&mut self, idx: usize) {
        self.marker = idx;
    }

    /// Returns the character currently pointed to by `self.idx`.
    fn current(&self) -> char {
        self.src[self.idx]
    }

    /// Parses the input into a TOMLDocument
    /// CLEANUP
    pub fn parse(&mut self) -> TOMLDocument {
        let mut body = Vec::new();

        // Take all keyvals outside of tables/AoT's
        while self.idx != self.end {
            // Break out when a table is found
            if self.current() == '[' {
                break;
            }
            // Take and wrap one KV pair
            body.push(self.parse_TLV());
        }

        // Switch to parsing tables and arrays of tables
        while self.idx != self.end {
            let next = self.dispatch_table();
            body.push(next);
        }

        TOMLDocument(body)
    }

    pub fn dispatch_table(&mut self) -> TLV {
        match self.current() {
            '[' if self.src[self.idx + 1] == '[' => {
                self.parse_AoT()
            }
            '[' => {
                self.parse_table(false)
            }
            _ => panic!("Should not have entered dispatch_table()"),
        }
    }

    /// Parses shallow AoTs
    pub fn parse_AoT(&mut self) -> TLV {
        let mut payload = Vec::new();
        let name = self.extract_AoT_name();
        
        while self.extract_AoT_name() == name {
            payload.push(self.parse_table(true));
        }

        TLV::AoT(payload)
    }

    pub fn extract_AoT_name(&mut self) -> Option<String> {
        println!("Made it into paotn");
        let start = self.idx;
        
        let res = match self.current() {
            '[' if self.src[self.idx+1] == '[' => {
                // Skip [[
                self.idx += 2;
                self.mark();
                
                while self.src[self.idx..self.idx+2] != [']', ']'] {
                    self.idx += 1;
                }
                Some(self.extract())
            }
            _ => None,
        };

        self.idx = start;
        res
    }

    pub fn parse_TLV(&mut self) -> TLV {
        // Mark start of whitespace
        self.mark();
        loop {
            match self.current() {
                // Found a newline; Return all whitespace found up to this point.
                // TODO: merge consecutive WS
                '\n' => {
                    self.idx += 1;
                    return TLV::WS(self.extract());
                }
                // Non line-ending ws, skip.
                ' ' | '\t' | '\r' => self.idx += 1,
                // Found a comment, parse it
                '#' => {
                    self.idx = self.marker;
                    let (mut c, trail) = self.parse_comment();
                    c.comment += &trail;
                    return TLV::Comment(c);
                }
                _ => {
                    // Return to begining of whitespace so it gets included
                    // as indentation into the value about to be parsed
                    self.idx = self.marker;
                    return TLV::Val(self.parse_key_value(true));
                }
            }
        }
    }

    /// Advances the parser to the first non-whitespce character
    /// and returns the consumed whitespace as a string.
    pub fn take_ws(&mut self) -> TLV {
        self.mark();
        while self.current().is_ws() {
            self.idx += 1;
            if self.src[self.idx] == '\n' {
                self.idx += 1;
                break;
            }
        }

        TLV::WS(self.extract())
    }

    /// Parses and returns a key/value pair.
    pub fn parse_key_value(&mut self, parse_comment: bool) -> KeyValue {
        self.mark();
        while self.src[self.idx].is_whitespace() {
            self.idx += 1;
        }

        let indent = self.extract();

        let key = match self.src[self.idx] {
            '"' => self.parse_quoted_key(),
            _ => self.parse_bare_key(),
        };

        // Skip = and whitespace
        while self.src[self.idx].is_ws_or_equal() {
            self.idx += 1;
        }

        // Parse value
        let val = self.parse_val();

        // Parse comment
        // TODO: Remove

        if !parse_comment || self.idx == self.end {
            if self.idx == self.end {
                println!("Reached EOF in comment parsing");
            }
            return KeyValue {
                indent: indent,
                key: key,
                value: val,
                comment: None,
                trail: "".to_string(),
            };
        } else {
            self.mark();
            while self.idx != self.src.len() - 1 && self.current() != '#' && self.current() != '\r' &&
                self.current() != '\n' {
                self.idx += 1;
            }

            let (comment, trailing) = match self.current() {
                '#' => {
                    self.idx = self.marker;
                    let (c, t) = self.parse_comment();
                    (Some(c), t)
                }
                '\r' => {
                    if self.src[self.idx + 1] == '\n' {
                        self.idx += 2;
                        // TODO: Check for out of bounds
                        let t = self.extract();
                        (None, t)
                    } else {
                        panic!("Invalid newline pattern");
                    }
                }
                '\n' => {
                    let t = self.src[self.marker..self.idx + 1].iter().cloned().collect::<String>();
                    (None, t)
                }
                // Then we reached EOF
                _ => {
                    let t = self.src[self.marker..self.idx + 1].iter().cloned().collect::<String>();
                    (None, t)
                }
            };

            KeyValue {
                indent: indent,
                key: key,
                value: val,
                comment: comment,
                trail: trailing,
            }
        }
    }

    /// Attempts to parse a value at the current position.
    pub fn parse_val(&mut self) -> Value {
        use self::Value::*;
        self.mark();
        match self.src[self.idx] {
            '"' if (self.src[self.idx+1] == '"' && self.src[self.idx+2] == '"') => {
                // skip """
                self.idx += 3;
                let mut lstart = self.idx;
                let mut actual = String::new();

                while self.src[self.idx..self.idx+3] != ['"', '"', '"'] {
                    match self.current() {
                        '/' if self.src[self.idx+1] == '\r' || self.src[self.idx+1] == '\n' => {
                            if lstart != self.idx {
                                let line = self.src[lstart..self.idx].iter().cloned().collect::<String>();
                                actual.push_str(&line);
                            }
                            self.idx += 1;
                            while self.current().is_ws() {
                                self.idx += 1;
                            }
                            lstart = self.idx;
                        }
                        _ => self.idx += 1,
                    }
                }
                self.idx += 2;
                let raw = self.extract();
                Str(StrEnum::MLBString(MLString{
                    actual: actual,
                    raw: raw,
                }))

            }
            '"' => {
                // skip '"' and mark
                self.idx += 1;
                self.mark();

                while self.src[self.idx] != '"' {
                    self.idx += 1;
                    if self.idx == self.src.len() {
                        println!("Single line string failure {:?}", &self.src[self.marker..]);
                    }
                }
                let payload = self.extract();
                // Clear '"'
                self. idx += 1;
                Str(StrEnum::SLBString(payload))
            }
            '\'' if (self.src[self.idx+1] == '\'' && self.src[self.idx+2] == '\'') => {
                // Skip '''
                self.idx += 3;
                self.mark();

                while self.src[self.idx..self.idx+3] != ['\'', '\'', '\''] {
                    self.idx += 1;
                }
                let payload = self.extract();
                self.idx += 3;
                Str(StrEnum::MLLString(payload))

            }
            '\'' => {
                // Skip '
                self.idx += 1;
                self.mark();

                while self.current() != '\'' {
                    self.idx += 1;
                }
                let payload  = self.extract();
                self.idx += 1;
                Str(StrEnum::SLLString(payload))
            }
            't' if self.src[self.idx..self.idx + 4] == ['t', 'r', 'u', 'e'] => {
                self.idx += 4;
                Bool(true)
            }
            'f' if self.src[self.idx..self.idx + 5] == ['f', 'a', 'l', 's', 'e'] => {
                self.idx += 5;
                Bool(false)
            }
            '[' => {
                // Create empty vec and skip '['
                let mut elems: Vec<Value> = Vec::new();
                self.idx += 1;

                while self.src[self.idx] != ']' {
                    while self.src[self.idx].is_ws() || self.src[self.idx] == ',' {
                        self.idx += 1;
                    }
                    let val = self.parse_val();
                    // self.idx += 1;
                    let check = val.discriminant();
                    elems.push(val);
                    assert_eq!(elems[0].discriminant(), check);
                }
                self.idx += 1;
                Array(elems)
            }
            '{' => {
                let mut elems: Vec<KeyValue> = Vec::new();
                self.idx += 1;

                while self.src[self.idx] != '}' {
                    while self.src[self.idx].is_ws() || self.current() == ',' {
                        self.idx += 1;
                    }
                    let val = self.parse_key_value(false);
                    elems.push(val);
                }
                if self.idx != self.end {
                    self.idx += 1;
                } else {
                    println!("Reached EOF in inline table parsing");
                }
                InlineTable(elems)
            }
            // TODO: Try parse int => float => datetime
            '+' | '-' | '0'...'9' => {
                // TODO: Really need capped integers...
                // Send help.
                while self.idx != self.src.len() - 1 &&
                      self.src[self.idx + 1].not_whitespace_or_pound() &&
                      self.src[self.idx + 1] != ',' &&
                      self.src[self.idx + 1] != ']' &&
                      self.src[self.idx + 1] != '}' {
                    self.idx += 1;
                }

                // TODO: Filtermap and why **?
                let clean: String = self.src[self.marker..self.idx + 1]
                    .iter()
                    .filter(|c| **c != '_')
                    .cloned()
                    .collect::<String>();

                // Skip last character of value being parsed
                self.idx += 1;

                // Ask forgiveness, not permission
                if let Ok(res) = i64::from_str(&clean) {
                    return Integer(res);
                } else if let Ok(res) = f64::from_str(&clean) {
                    return Float(res);
                } else if let Ok(res) = ChronoDateTime::parse_from_rfc3339(&clean) {
                    return DateTime((res, clean));
                }

                println!("working on: {:?}", clean);
                panic!("Could not parse to int, float or DateTime");
            }
            _ => {
                println!("Current: {}", self.current());
                panic!("Could not infer type of value being parsed");
            }
        }
    }

    /// Attempts to parse a comment at the current position, and returns it along with
    /// the newline character. Only call this function if the presence of the pound sign
    ///  is guaranteed.
    // TODO: WTF is "trailing"?
    fn parse_comment(&mut self) -> (Comment, String) {
        self.mark();

        // Find this comment's indentation w.r.t. the last non-ws character.
        while self.current() != '#' {
            self.idx += 1;
        }
        let indent = self.extract();

        // Skip #
        self.idx += 1;
        self.mark();

        // The comment itself
        while self.current() != '\r' && self.current() != '\n' {
            self.idx += 1;
        }
        let comment = self.extract();

        self.mark();
        let trailing = match self.current() {
            '\r' => {
                self.idx += 2;
                "\r\n".to_string()
            }
            '\n' => {
                self.idx += 1;
                "\n".to_string()
            }
            _ => unreachable!(),
        };

        (Comment {
            indent: indent,
            comment: comment,
        },
         trailing)

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

        self.extract()
    }

    pub fn parse_quoted_key(&mut self) -> Key {
        // Skip "
        self.idx += 1;
        self.mark();

        while self.src[self.idx] != '"' {
            self.idx += 1;
        }

        let key = self.extract();
        // Skip "
        self.idx += 1;

        Key::Quoted(key)
    }

    pub fn parse_bare_key(&mut self) -> Key {
        self.mark();
        while self.src[self.idx].is_bare_key_char() {
            self.idx += 1;
        }
        let key = self.extract();
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

    // TODO: Clean this for the love of Eru
    pub fn parse_table(&mut self, array: bool) -> TLV {
        // Lands on '[' character, skip it.
        let inc = match array {
            false => 1,
            true => 2,
        };
        self.idx += inc;
        self.mark();

        // Seek the end of the table's name
        while self.current() != ']' {
            // TODO: Quoted names
            self.idx += 1;
        }
        
        // Get the name
        let name = self.extract();
        // println!("{}", name);

        // FRAGILE: Seek start of next line
        while self.current() != '\n' {
            self.idx += 1;
        }
        self.idx += 1;

        let mut values = Vec::new();
        loop {
            if self.idx == self.end {
                break;
            }

            match self.current() {
                '[' => break,
                _ => values.push(self.parse_TLV()),
            }
        }

        TLV::Table(Table {
            array: array,
            name: vec![name],
            comment: "".to_string(),
            values: values,
        })

    }
}