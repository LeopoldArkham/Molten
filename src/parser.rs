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
        Parser {
            src: input.chars().collect::<Vec<char>>(),
            idx: 0,
            marker: 0,
            end: input.len() - 1 as usize,
        }
    }

    /// Extract the value between marker and index.
    fn extract(&self) -> String {
        let end = if self.not_end() {
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

    fn not_end(&self) -> bool {
        self.idx != self.end
    }

    fn end(&self) -> bool {
        !self.not_end()
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
        child != parent && child.starts_with(parent)
    }

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
                // EOF ws
                ' ' | '\t' if self.end() => {
                    return (Item::WS(self.extract()), None);
                }
                // Non line-ending ws, skip.
                ' ' | '\t' | '\r' => self.idx += 1,
                // Found a comment, parse it
                '#' => {
                    self.idx = self.marker;
                    let (c, trail) = self.parse_comment_trail();
                    let mut c = c.expect("There really should be a comment here - parse_item()");
                    c.comment += &trail;
                    return (Item::Comment(c), None);
                }
                '[' => {
                    println!("Found table!");
                    self.idx = self.marker;
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

    /// Attempts to parse a comment at the current position, and returns it along with
    /// the newline character. Only call this function if the presence of the pound sign
    /// is guaranteed.
    fn parse_comment(&mut self) -> Comment {
        // Find this comment's indentation w.r.t. the last non-ws character.
        self.mark();
        while self.current() != '#' {
            self.idx += 1;
        }

        let indent = self.extract();
        // Skip #
        self.idx += 1;
        self.mark();

        // The comment itself
        // @fixme: Comment on EOF
        while self.not_end() && !self.current().is_nl() {
            self.idx += 1;
        }

        let comment = self.extract();
        Comment {
            indent: indent,
            comment: comment,
        }
    }

    pub fn parse_comment_trail(&mut self) -> (Option<Comment>, String) {
        let mut comment = None;
        self.mark();

        while !self.current().is_nl() {
            if self.current() == '#' {
                self.idx = self.marker;
                comment = Some(self.parse_comment()); // Ends on first NL or last char of comment if EOF
                self.mark();
                break;
            }
            if !self.inc() {
                break;
            }
        }
        while !self.current().is_nl() && self.inc() {}
        if self.current() == '\r' {self.inc();}
        if self.current() == '\n' {self.inc();}

        let trail = if self.idx != self.marker || self.current().is_ws() {
            self.extract()
        } else {
            "".to_string()
        };
        (comment, trail)
    }

    /// Parses and returns a key/value pair.
    pub fn parse_key_value(&mut self, parse_comment: bool) -> (Item, Option<Key>) {
        self.mark();

        // Extract indentation
        while self.src[self.idx].is_whitespace() {
            self.idx += 1;
        }
        let indent = self.extract();

        // Dispatch on key type
        // @cleanup: Separate function
        let key = match self.src[self.idx] {
            '"' => self.parse_quoted_key(),
            _ => self.parse_bare_key(),
        };

        // Skip = and whitespace
        // @incomplete: Extract for full KV reproduction
        while self.src[self.idx].is_ws_or_equal() {
            self.idx += 1;
        }

        // Parse value
        let mut val = self.parse_val();
        val.meta_mut().indent = indent;

        // Handle end of line
        if parse_comment {
            let (comment, trail) = self.parse_comment_trail();
            val.meta_mut().comment = comment;
            val.meta_mut().trail = trail;
        }
        (val, Some(key))
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
                while self.src[self.idx] != '"' {
                    self.idx += 1;
                    if self.idx == self.src.len() {
                        println!("Single line string failure {:?}", &self.src[self.marker..]);
                    }
                }

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
                // @incomplete: Must allow comments here as well
                // Move comment branching logic in parse_val?

                // Create empty vec and skip '['
                let mut elems: Vec<Item> = Vec::new();
                self.inc();

                while self.current() != ']' {
                    // WS and separators being skipped here
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
                    elems.push(self.parse_val());
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
                    println!("Parsing KV in inline table: {:?} {:?}", key, val);

                    let _ = elems.append(key, val).map_err(|e| panic!(e.to_string()));
                }
                // @knob
                self.inc();
                Item::InlineTable {
                    val: elems,
                    meta: meta,
                }
            }
            // Integer, Float, or DateTime
            '+' | '-' | '0'...'9' => {
                // @cleanup
                while self.current().not_whitespace_or_pound() && self.current() != ',' &&
                      self.current() != ']' && self.current() != '}' &&
                      self.inc() {}
                // EOF shittiness
                match self.current() {
                    '0'...'9' => {}
                    _ => {
                        self.idx -= 1;
                    }
                }
                let raw = self.extract_inclusive();
                self.inc();

                let clean: String = raw.chars()
                    .filter(|c| *c != '_' && *c != ' ')
                    .collect::<String>();

                // Ask forgiveness, not permission
                if let Ok(res) = i64::from_str(&clean) {
                    return Item::Integer {
                        val: res,
                        meta: meta,
                        raw: raw,
                    };
                } else if let Ok(res) = f64::from_str(&clean) {
                    // @incomplete: "Similar to integers, you may use underscores to enhance readability.
                    // Each underscore must be surrounded by at least one digit."
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
                // println!("Current: {}",
                        //  self.src[self.idx..].iter().collect::<String>());
                panic!("Could not infer type of value being parsed");
            }
        }
    }

    // @incomplete: Does straight up nothing
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

    // @cleanup: Old code
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

    // @cleanup
    // @todo: change return parameter order to match parse_item()
    pub fn parse_table(&mut self, array: bool) -> (Key, Item) {
        let mut values = Container::new();

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
            // @todo: Quoted names
            self.idx += 1;
        }
        let name = self.extract_exact();
        let key = Key {
            t: KeyType::Bare,
            raw: name.clone(),
            actual: name.clone(),
        };
        // ENDS ON FIRST "]"
        // --------------------------

        // Get comment and trail
        let mut comment: Option<Comment> = None;
        let mut trail = "".to_string();

        if array {
            self.inc();
        }

        if self.end() {
            return (key,
                    Item::Table {
                is_array: array,
                val: values,
                meta: LineMeta {
                    indent: indent,
                    comment: comment,
                    trail: trail,
                },
            });
        }

        self.inc();
        self.mark();

        // Search for a comment until a newline is found
        // @cleanup: Compare with comment seeking logic in parse_key_value
        let rewind = self.idx;

        while !self.current().is_nl() {
            if self.current() == '#' {
                self.idx = rewind;
                let c = self.parse_comment_trail();
                comment = c.0;
                trail = c.1;
                break;
            }
            if !self.inc() {
                self.idx = rewind;
                break;
            }
        }

        if comment.is_none() {
            while self.current() != '\n' && self.inc() {
            }
            trail = self.extract_inclusive();
            self.inc();
        }

        // --------------------------
        
        // Parse content
        loop {
            match self.current() {
                '[' => {
                    // @incomplete: "next" could be aot
                    // @fixme: snippet repeated below
                    let name_next = self.peek_table_name();
                    match self.is_child(&name, &name_next) {
                        true => {
                            let next = self.dispatch_table();
                            let _ = values.append(next.1, next.0)
                                .map_err(|e| panic!(e.to_string()));
                        }
                        false => break,
                    }
                }
                ' ' | '\t' => {
                    self.mark();
                    while self.current().is_ws() {
                        if !self.inc() {
                            break;
                        }
                    }
                    match self.current() {
                        '[' => {
                            // @incomplete: "next" could be aot
                            let name_next = self.peek_table_name();
                            match self.is_child(&name, &name_next) {
                                true => {
                                    let next = self.dispatch_table();
                                    let _ = values.append(next.1, next.0)
                                        .map_err(|e| panic!(e.to_string()));
                                }
                                false => break,
                            }
                        }
                        _ => {
                            self.idx = self.marker;
                            let kv = self.parse_item();
                            let _ = values.append(kv.0, kv.1).map_err(|e| panic!(e.to_string()));
                        }
                    }
                }
                _ if self.current() != '\n' => {
                    let kv = self.parse_item();
                    let _ = values.append(kv.0, kv.1).map_err(|e| panic!(e.to_string()));
                }
                _ => {}
            }
            if self.idx == self.end {
                break;
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
