use tomldoc::TOMLDocument;
use tomlchar::TOMLChar;
use items::*;
use comment::Comment;
use container::Container;

use chrono::{DateTime as ChronoDateTime};

use std::str::FromStr;

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

    /// Increments the parser if the end of the input has not been reached
    fn inc(&mut self) -> bool {
        if self.idx != self.end {
            self.idx += 1;
            true
        } else {
            false
        }
    }

    /// Sets the marker to the index's current position
    fn mark(&mut self) {
        self.marker = self.idx;
    }

    /// Returns the character currently pointed to by `self.idx`.
    fn current(&self) -> char {
        self.src[self.idx]
    }

    /// Parses the input into a TOMLDocument
    /// CLEANUP
    pub fn parse(&mut self) -> TOMLDocument {
        let mut body = Container::new();

        // Take all keyvals outside of tables/AoT's
        while self.idx != self.end {
            // Break out when a table is found
            if self.current() == '[' {
                break;
            }
            // Take and wrap one KV pair
            let kv = self.parse_item();
            let _ = body.append(kv.0, kv.1).map_err(|e| panic!(e.to_string()));
        }

        // Switch to parsing tables and arrays of tables
        while self.idx != self.end {
            let (k, v) = self.dispatch_table();
            let _ = body.append(v, k).map_err(|e| panic!(e.to_string()));
        }

        TOMLDocument(body)
    }

    pub fn dispatch_table(&mut self) -> (Key, Item) {
        match self.current() {
            '[' if self.src[self.idx + 1] == '[' => self.parse_AoT(),
            '[' => self.parse_table(false),
            _ => panic!("Should not have entered dispatch_table()"),
        }
    }

    fn is_child(&self, parent: &str, child: &str) -> bool {
        false
    }
 /*
    // MOCK
    fn wishlist(&mut self) {
        // With body: Container
        // Switch to table-like parsing
        let next = self.dispatch_table();
        // Item is not aware of its name
        let key_next = next.0.normalize();

        // Get latest item's key
        let key_prev = body.last_item(); // Operates recursively until a container is reached whose last value is not a container 

        match self.is_child(key_prev, key_next) {
            true =>
            false => 
        }
    }
*/
    #[allow(non_snake_case)]
    /// Parses shallow AoTs
    pub fn parse_AoT(&mut self) -> (Key, Item) {
        let mut payload = Vec::new();
        let name = self.peek_table_name();
        while self.peek_table_name() == name {
            payload.push(self.parse_table(true).1);
        }
        let key = Key {
            t: KeyType::Bare,
            raw: name.clone(),
            actual: name,
        };
        (key, Item::AoT(payload))
    }

    /// Peeks at a table-like element, returning its name,
    /// and resetting the parser's position.
    fn peek_table_name(&mut self) -> String {
        let rewind = self.idx;

        while self.current() != '[' {
            println!("Skipping ws before '[' in peek table name");
            self.idx += 1;
        }

        while self.current() == '[' {
            self.idx += 1;
        }

        self.mark();

        while self.current() != ']' {
            self.idx += 1;
        }

        let r = self.extract();
        self.idx = rewind;
        r
    }

    /// Attempts to parse the next item and returns it, along with its key
    /// if the item is value-like.
    pub fn parse_item(&mut self) -> (Item, Option<Key>) {
        // Mark start of whitespace
        self.mark();
        loop {
            match self.current() {
                // Found a newline; Return all whitespace found up to this point.
                // TODO: merge consecutive WS
                '\n' => {
                    self.idx += 1;
                    return (Item::WS(self.extract()), None);
                }
                // Non line-ending ws, skip.
                ' ' | '\t' | '\r' => self.idx += 1,
                // Found a comment, parse it
                '#' => {
                    self.idx = self.marker;
                    let (mut c, trail) = self.parse_comment();
                    c.comment += &trail;
                    return (Item::Comment(c), None);
                }
                '[' => {
                    // self.idx = self.marker;
                    let r = self.dispatch_table();
                    return (r.1, Some(r.0));
                }
                _ => {
                    // Return to begining of whitespace so it gets included
                    // as indentation into the value about to be parsed
                    self.idx = self.marker;
                    return self.parse_key_value(true);
                }
            }
        }
    }

    /// Parses and returns a key/value pair.
    pub fn parse_key_value(&mut self, parse_comment: bool) -> (Item, Option<Key>) {
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
        let mut val = self.parse_val();
        val.meta_mut().indent = indent;
        // Parse comment
        // TODO: Remove

        if !parse_comment || self.idx == self.end {
            if self.idx == self.end {
                //lolwut
            }
            return (val, Some(key));
        } else {
            // SEND HELP
            self.mark();
            while self.idx != self.src.len() - 1 && self.current() != '#' &&
                  self.current() != '\r' && self.current() != '\n' {
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
            val.meta_mut().comment = comment;
            val.meta_mut().trail = trailing;
            (val, Some(key))
        }
    }

    /// Attempts to parse a value at the current position.
    pub fn parse_val(&mut self) -> Item {
        self.mark();
        let meta: LineMeta = Default::default();
        match self.src[self.idx] {
            '"' if (self.src[self.idx + 1] == '"' && self.src[self.idx + 2] == '"') => {
                // skip """
                self.idx += 3;
                let mut lstart = self.idx;
                let mut actual = String::new();

                while self.src[self.idx..self.idx + 3] != ['"', '"', '"'] {
                    match self.current() {
                        '/' if self.src[self.idx + 1] == '\r' || self.src[self.idx + 1] == '\n' => {
                            if lstart != self.idx {
                                let line =
                                    self.src[lstart..self.idx].iter().cloned().collect::<String>();
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

                Item::Str {
                    t: StringType::MLB(raw),
                    val: actual,
                    meta: meta,
                }
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
                self.idx += 1;

                Item::Str {
                    t: StringType::SLB,
                    val: payload,
                    meta: meta,
                }
            }
            '\'' if (self.src[self.idx + 1] == '\'' && self.src[self.idx + 2] == '\'') => {
                // Skip '''
                self.idx += 3;
                self.mark();

                while self.src[self.idx..self.idx + 3] != ['\'', '\'', '\''] {
                    self.idx += 1;
                }
                let payload = self.extract();
                self.idx += 3;

                Item::Str {
                    t: StringType::MLL,
                    val: payload,
                    meta: meta,
                }
            }
            '\'' => {
                // Skip '
                self.idx += 1;
                self.mark();

                while self.current() != '\'' {
                    self.idx += 1;
                }
                let payload = self.extract();
                self.idx += 1;

                Item::Str {
                    t: StringType::SLL,
                    val: payload,
                    meta: meta,
                }
            }
            't' if self.src[self.idx..self.idx + 4] == ['t', 'r', 'u', 'e'] => {
                self.idx += 4;

                Item::Bool {
                    val: true,
                    meta: meta,
                }
            }
            'f' if self.src[self.idx..self.idx + 5] == ['f', 'a', 'l', 's', 'e'] => {
                self.idx += 5;

                Item::Bool {
                    val: false,
                    meta: meta,
                }
            }
            '[' => {
                // Create empty vec and skip '['
                let mut elems: Vec<Item> = Vec::new();
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
                Item::Array {
                    val: elems,
                    meta: meta,
                }
            }
            '{' => {
                let mut elems: Container = Container::new();
                self.idx += 1;

                while self.src[self.idx] != '}' {
                    while self.src[self.idx].is_ws() || self.current() == ',' {
                        self.idx += 1;
                    }
                    let (key, val) = self.parse_key_value(false);
                    let _ = elems.append(key, val).map_err(|e| panic!(e.to_string()));
                }
                if self.idx != self.end {
                    self.idx += 1;
                }
                Item::InlineTable {
                    val: elems,
                    meta: meta,
                }
            }
            '+' | '-' | '0'...'9' => {
                // TODO: Clean this mess ffs
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
                    return Item::Integer {
                        val: res,
                        meta: meta,
                    };
                } else if let Ok(res) = f64::from_str(&clean) {
                    return Item::Float {
                        val: res,
                        meta: meta,
                    };
                } else if let Ok(res) = ChronoDateTime::parse_from_rfc3339(&clean) {
                    return Item::DateTime {
                        val: res,
                        raw: clean,
                        meta: meta,
                    };
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
    /// is guaranteed.
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

        Key {
            t: KeyType::Quoted,
            actual: key.clone(),
            raw: key,
        }
    }

    pub fn parse_bare_key(&mut self) -> Key {
        self.mark();
        while self.src[self.idx].is_bare_key_char() {
            self.idx += 1;
        }
        let key = self.extract();

        Key {
            t: KeyType::Bare,
            actual: key.clone(),
            raw: key,
        }
    }

    // TODO: Clean this for the love of Eru
    pub fn parse_table(&mut self, array: bool) -> (Key, Item) {
        // Extract indent if any
        while self.src[self.idx - 1] != '\n' {
            self.idx -= 1;
        }
        self.mark();
        while self.current().is_ws() {
            self.idx += 1;
        }
        let indent = self.extract();
        // -------------------------

        // Extract the name into a key
        let inc = match array {
            false => 1,
            true => 2,
        };
        self.idx += inc;
        self.mark();
        while self.current() != ']' {
            // TODO: Quoted names
            self.idx += 1;
        }
        let name = self.extract();
        let key = Key {
            t: KeyType::Bare,
            raw: name.clone(),
            actual: name,
        };
        // --------------------------

        // Get comment and trail
        self.idx += inc;
        self.mark();

        let mut comment: Option<Comment> = None;
        let mut trail = "".to_string();
        while !self.current().is_nl() {
            if self.current() == '#' {
                let r = self.parse_comment();
                comment = Some(r.0);
                trail = r.1;
                break;
            }
            self.idx += 1;
        }
        if comment.is_none() {
            while self.current().is_nl() {
                self.idx += 1
            }
            trail = self.extract();
        }
        // --------------------------

        // Parse content
        let mut values = Container::new();
        loop {
            if self.idx == self.end {
                break;
            }

            match self.current() {
                '[' => break,
                ' ' | '\t' => {
                    self.mark();
                    while self.current().is_ws() {
                        self.idx += 1;
                    }
                    match self.current() {
                        '[' => break,
                        _ => {
                            self.idx = self.marker;
                            let kv = self.parse_item();
                            let _ = values.append(kv.0, kv.1).map_err(|e| panic!(e.to_string()));
                        }
                    }
                }
                _ => {
                    let kv = self.parse_item();
                    let _ = values.append(kv.0, kv.1).map_err(|e| panic!(e.to_string()));
                }
            }
        }
        (key,
         Item::Table {
            is_array: array,
            val: values,
            meta: LineMeta {
                indent: indent,
                comment: comment,
                trail: trail,
            },
        })
    }
}