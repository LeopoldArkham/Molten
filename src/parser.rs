use tomldoc::TOMLDocument;
use tomlchar::TOMLChar;
use items::*;
use comment::Comment;
use container::Container;

use chrono::DateTime as ChronoDateTime;

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
        let src = input.chars().collect::<Vec<char>>();
        let end = if src.is_empty() {
            0
        } else {
            src.len() - 1
        };
        Parser {
            src: src,
            idx: 0,
            marker: 0,
            end: end,
        }
    }

    /// Extract the value between marker and index.
    fn extract(&self) -> String {
        let end = if !self.end() {
            self.idx
        } else {
            self.idx + 1
        };
        self.src[self.marker..end].iter().cloned().collect::<String>()
    }

    fn extract_exact(&mut self) -> String {
        self.src[self.marker..self.idx].iter().cloned().collect::<String>()
    }

    fn extract_inclusive(&mut self) -> String {
        self.src[self.marker..self.idx + 1].iter().cloned().collect::<String>()
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

    fn end(&self) -> bool {
        self.idx >= self.end
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
    /// @cleanup: conflicts with parse_item wrt table parsing
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

        // Switch to parsing tables/arrays of tables
        while self.idx != self.end {
            let (k, v) = self.dispatch_table();
            let _ = body.append(k, v).unwrap();
        }

        TOMLDocument(body)
    }

    pub fn dispatch_table(&mut self) -> (Key, Item) {
        while self.current().is_spaces() && self.inc() {}
        match self.current() {
            '[' if self.src[self.idx + 1] == '[' => self.parse_AoT(),
            '[' => self.parse_table(),
            _ => {
                panic!("Should not have entered dispatch_table()");
            }
        }
    }

    fn is_child(parent: &str, child: &str) -> bool {
        child != parent && child.starts_with(parent)
    }

    #[allow(non_snake_case)]
    /// Parses AoTs
    pub fn parse_AoT(&mut self) -> (Key, Item) {
        let mut array = Vec::new();
        let (key, first) = self.parse_table();
        array.push(first);

        while !self.end() {
            let rewind = self.idx;
            let (k, table) = self.parse_table();
            if key.as_string() == k.as_string() {
                array.push(table);
            } else {
                self.idx = rewind;
                break;
            }
        }
        (key, Item::AoT(array))
    }

    /// Attempts to parse the next item and returns it, along with its key
    /// if the item is value-like.
    pub fn parse_item(&mut self) -> (Option<Key>, Item) {
        // Mark start of whitespace
        self.mark();
        loop {
            match self.current() {
                // Found a newline; Return all whitespace found up to this point.
                // TODO: merge consecutive WS
                '\n' => {
                    self.idx += 1;
                    return (None, Item::WS(self.extract()));
                }
                // EOF ws
                ' ' | '\t' if self.end() => {
                    return (None, Item::WS(self.extract()));
                }
                // Non line-ending ws, skip.
                ' ' | '\t' | '\r' => self.idx += 1,
                // Found a comment, parse it
                '#' => {
                    self.idx = self.marker;
                    let (c, trail) = self.parse_comment_trail();
                    let mut c = c.expect("There really should be a comment here - parse_item()");
                    c.comment += &trail;
                    return (None, Item::Comment(c));
                }
                '[' => {
                    let r = self.dispatch_table();
                    return (r.0.into(), r.1);
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

    /// Attempts to parse a comment at the current position, and returns it along with
    /// the newline character. Only call this function if the presence of the pound sign
    /// is guaranteed.
    fn parse_comment(&mut self) -> Comment {
        // Find this comment's indentation w.r.t. the last non-ws character.
        self.mark();
        while self.current() != '#' && self.inc() {}

        let indent = self.extract();
        // Skip #
        self.idx += 1;
        self.mark();

        // The comment itself
        while !self.end() && !self.current().is_nl() && self.inc() {}

        let comment = self.extract();
        Comment {
            indent: indent,
            comment: comment,
        }
    }

    pub fn parse_comment_trail(&mut self) -> (Option<Comment>, String) {
        let mut comment = None;
        let mut trail = "".to_string();

        if self.end() {
            return (comment, trail);
        }
        self.mark();
        loop {
            match self.current() {
                '\n' => break,
                '#' => {
                    self.idx = self.marker;
                    comment = Some(self.parse_comment()); // Ends on first NL or last char if EOF
                    self.mark();
                    break;
                }
                ' ' | '\t' | '\r' | ',' => {
                    self.inc();
                }
                _ => break,
            }
            if self.end() {
                break;
            }
        }
        while self.current().is_spaces() && self.inc() {}
        if self.current() == '\r' {
            self.inc();
        }
        if self.current() == '\n' {
            self.inc();
        }

        trail = if self.idx != self.marker || self.current().is_ws() {
            self.extract()
        } else {
            "".to_string()
        };
        (comment, trail)
    }

    /// Parses and returns a key/value pair.
    pub fn parse_key_value(&mut self, parse_comment: bool) -> (Option<Key>, Item) {
        self.mark();
        while self.current().is_spaces() && self.inc() {}
        let indent = self.extract();

        let mut key = self.parse_key();
        self.mark();
        while self.current().is_kv_sep() && self.inc() {}
        key.sep = self.extract_exact();

        let mut val = self.parse_val();

        if parse_comment {
            let (comment, trail) = self.parse_comment_trail();
            val.meta_mut().comment = comment;
            val.meta_mut().trail = trail;
        }
        val.meta_mut().indent = indent;

        (Some(key), val)
    }

    /// Attempts to parse a value at the current position.
    pub fn parse_val(&mut self) -> Item {
        self.mark();
        let meta: LineMeta = Default::default();
        match self.current() {
            // Multi Line Basic String
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
                        _ => {
                            self.inc();
                        }
                    }
                }
                self.inc();
                self.inc();
                self.inc();
                let raw = self.extract();

                Item::Str {
                    t: StringType::MLB(raw),
                    val: actual,
                    meta: meta,
                }
            }
            // Single Line Basic String
            '"' => {
                // skip '"' and mark
                self.idx += 1;
                self.mark();

                // @incomplete: Needs to account for escaped backslashes
                // Seek end of string
                while self.current() != '"' && self.inc() {}

                let payload = self.extract_exact();
                // Clear '"'
                self.inc();

                Item::Str {
                    t: StringType::SLB,
                    val: payload,
                    meta: meta,
                }
            }
            // Multi Line literal String
            '\'' if (self.src[self.idx + 1] == '\'' && self.src[self.idx + 2] == '\'') => {
                // Skip '''
                self.idx += 3;
                self.mark();

                while self.src[self.idx..self.idx + 3] != ['\'', '\'', '\''] {
                    self.idx += 1;
                }
                let payload = self.extract();
                // Two slashes guaranteed
                self.idx += 2;
                self.inc();

                Item::Str {
                    t: StringType::MLL,
                    val: payload,
                    meta: meta,
                }
            }
            // Single Line literal String
            '\'' => {
                // Skip '
                self.idx += 1;
                self.mark();

                while self.current() != '\'' {
                    self.idx += 1;
                }
                let payload = self.extract_exact();
                self.inc();

                Item::Str {
                    t: StringType::SLL,
                    val: payload,
                    meta: meta,
                }
            }
            // Boolean: true
            't' if self.src[self.idx..self.idx + 4] == ['t', 'r', 'u', 'e'] => {
                self.idx += 3;
                self.inc();

                Item::Bool {
                    val: true,
                    meta: meta,
                }
            }
            // Boolean: False
            'f' if self.src[self.idx..self.idx + 5] == ['f', 'a', 'l', 's', 'e'] => {
                self.idx += 4;
                self.inc();

                Item::Bool {
                    val: false,
                    meta: meta,
                }
            }
            // Array
            '[' => {
                let mut elems: Vec<Item> = Vec::new();
                self.inc();

                while self.current() != ']' {
                    self.mark();
                    while self.current().is_ws() || self.current() == ',' {
                        self.inc();
                    }
                    if self.idx != self.marker {
                        elems.push(Item::WS(self.extract_exact()));
                    }
                    if self.current() == ']' {
                        break;
                    }
                    let next = match self.current() {
                        '#' => Item::Comment(self.parse_comment()),
                        _ => self.parse_val(),
                    };
                    elems.push(next);
                }
                self.inc();

                let res = Item::Array {
                    val: elems,
                    meta: meta,
                };

                if res.is_homogeneous() {
                    res
                } else {
                    panic!("Non homogeneous array");
                }
            }
            // Inline Table
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
                self.inc();

                Item::InlineTable {
                    val: elems,
                    meta: meta,
                }
            }
            // Integer, Float, or DateTime
            '+' | '-' | '0'...'9' => {
                while self.current().not_in(" \t\n\r#,]}") && self.inc() {}

                // EOF shittiness
                // @cleanup: rewrite with in() method on TOMLchar
                if !self.current().is_digit(10) {
                    self.idx -= 1;
                }

                let raw = self.extract_inclusive();
                self.inc();

                let clean: String = raw.chars()
                    .filter(|c| *c != '_' && *c != ' ')
                    .collect::<String>();

                // Forgiveness > Permission
                if let Ok(res) = i64::from_str(&clean) {
                    return Item::Integer {
                        val: res,
                        meta: meta,
                        raw: raw,
                    };
                } else if let Ok(res) = f64::from_str(&clean) {
                    // @incomplete: "Similar to integers, you may use underscores to enhance
                    // readability. Each underscore must be surrounded by at least one digit."
                    return Item::Float {
                        val: res,
                        meta: meta,
                        raw: raw,
                    };
                } else if let Ok(res) = ChronoDateTime::parse_from_rfc3339(&clean) {
                    return Item::DateTime {
                        val: res,
                        raw: clean,
                        meta: meta,
                    };
                }

                // @incomplete: Error management
                println!("working on: {:?}", raw);
                panic!("Could not parse to int, float or DateTime");
            }
            _ => {
                // @incomplete: Error management
                println!("Current: {}",
                         self.src[self.idx..].iter().collect::<String>());
                panic!("Could not infer type of value being parsed");
            }
        }
    }

    /// Parses a Key at the current position;
    /// WS before the key must be exhausted first at the callsite.
    fn parse_key(&mut self) -> Key {
        let key = match self.current() {
            '"' | '\'' => self.parse_quoted_key(),
            _ => self.parse_bare_key(),
        };
        key
    }

    fn parse_quoted_key(&mut self) -> Key {
        let quote_style = self.current();
        let key_type = match quote_style {
            '"' => KeyType::Basic,
            '\'' => KeyType::Literal,
            _ => panic!("Should not have entered parse_quoted_key()"),
        };
        self.inc();
        self.mark();

        while self.current() != quote_style && self.inc() {}
        let key = self.extract();
        self.inc();

        Key {
            t: key_type,
            sep: "".to_string(),
            key: key,
        }
    }

    fn parse_bare_key(&mut self) -> Key {
        self.mark();
        while self.current().is_bare_key_char() && self.inc() {}
        let key = self.extract();

        Key {
            t: KeyType::Bare,
            sep: "".to_string(),
            key: key,
        }
    }

    /// Rewinds parser to the beginning of the current line
    fn rewind(&mut self) {
        while self.idx != 0 && self.src[self.idx - 1] != '\n' {
            self.idx -= 1;
        }
    }

    pub fn parse_table(&mut self) -> (Key, Item) {
        // Indentation
        self.rewind();
        self.mark();
        while self.current().is_ws() && self.inc() {}
        let indent = self.extract();
        // -------------------------

        // Aot?
        self.inc();
        let is_array = if self.current() == '[' {
            self.inc();
            true
        } else {
            false
        };
        debug_assert_ne!(self.current(), '[');
        // -----------------------

        // Key
        self.mark();
        while self.current() != ']' && self.inc() {}

        let name = self.extract_exact();
        let key = Key {
            t: KeyType::Bare,
            sep: "".to_string(),
            key: name.to_string(),
        };
        self.inc();
        if is_array {
            self.inc();
        }
        // --------------------------

        let (comment, trail) = self.parse_comment_trail();

        let mut values = Container::new();
        // @todo: cache parsed tables instead of rewinding.
        // Maybe cache in a map by start index?
        while !self.end() {
            let rewind = self.idx;
            let (key, item) = self.parse_item();

            if item.is_table() && !Parser::is_child(&name, &key.as_ref().unwrap().as_string()) {
                self.idx = rewind;
                break;
            }
            let _ = values.append(key, item).map_err(|e| panic!(e.to_string()));
        }

        (key,
         Item::Table {
            is_array: is_array,
            val: values,
            meta: LineMeta {
                indent: indent,
                comment: comment,
                trail: trail,
            },
        })
    }
}
