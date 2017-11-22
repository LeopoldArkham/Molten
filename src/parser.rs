use tomldoc::TOMLDocument;
use tomlchar::TOMLChar;
use items::*;
use errors::*;
use container::Container;

use chrono::DateTime as ChronoDateTime;

use std::str::{FromStr, CharIndices};

// FIXME: Allowing dead code due to https://github.com/rust-lang/rust/issues/18290
#[allow(non_camel_case_types, dead_code)]
type isAOT = bool;

/// Parser for TOML documents.
#[derive(Debug)]
pub struct Parser<'a> {
    /// Input to parse.
    src: &'a str,
    /// Iterator used for getting characters from `src`.
    chars: CharIndices<'a>,
    /// Current byte offset into `src`.
    idx: usize,
    /// Current character
    current: char,
    /// Index into `src` between which and `idx` slices will be extracted
    marker: usize,
    /// A LIFO stack to keep track of the current AoT.
    AoT_stack: Vec<&'a str>,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a &str.
    pub fn new(input: &'a str) -> Parser<'a> {
        let mut p = Parser {
            src: input,
            chars: input.char_indices(),
            idx: 0,
            marker: 0,
            current: '\0',
            AoT_stack: Vec::new(),
        };
        p.inc();
        p
    }

    /// Extract the value between marker and index.
    fn extract(&mut self) -> &'a str {
        if self.end() {
            &self.src[self.marker..]
        } else {
            &self.src[self.marker..self.idx]
        }
    }

    // HACK: this should go away
    /// Extract the exact value between marker and index.
    fn extract_exact(&mut self) -> &'a str {
        &self.src[self.marker..self.idx]
    }

    /// Increments the parser if the end of the input has not been reached.
    /// Returns whether or not it was able to advance.
    fn inc(&mut self) -> bool {
        match self.chars.next() {
            Some((i, ch)) => {
                self.idx = i;
                self.current = ch;
                true
            }
            None => {
                self.idx = self.src.len();
                self.current = '\0';
                false
            }
        }
    }

    /// Returns true if the parser has reached the end of the input.
    fn end(&self) -> bool {
        self.idx >= self.src.len()
    }

    /// Sets the marker to the index's current position
    fn mark(&mut self) {
        self.marker = self.idx;
    }

    /// Create error at the current position.
    fn parse_error(&self) -> Error {
        // TODO: Actually report position of error (#14)
        let (line, col) = (0, 0);
        ErrorKind::ParseError(line, col).into()
    }

    /// Parses the input into a TOMLDocument
    pub fn parse(&mut self) -> Result<TOMLDocument<'a>> {
        let mut body = Container::new();

        // Take all keyvals outside of tables/AoT's.
        while !self.end() {
            // Break out if a table is found.
            if self.current == '[' {
                break;
            }
            // Otherwise, take and append one KV.
            if let Some((key, value)) = self.parse_item()? {
                body.append(key, value).chain_err(|| self.parse_error())?;
                self.mark();
            } else {
                break;
            }
        }

        // Switch to parsing tables/arrays of tables until the end of the input.
        while !self.end() {
            let (k, v) = self.parse_table()?;
            body.append(k, v).chain_err(|| self.parse_error())?;
        }
        Ok(TOMLDocument(body))
    }

    /// Returns whether a key is strictly a child of another key.
    /// AoT siblings are not considered children of one another.
    fn is_child(parent: &str, child: &str) -> bool {
        child != parent && child.starts_with(parent)
    }

    /// Attempts to parse the next item and returns it, along with its key
    /// if the item is value-like.
    pub fn parse_item(&mut self) -> Result<Option<(Option<Key<'a>>, Item<'a>)>> {
        // Mark start of whitespace
        self.mark();
        loop {
            match self.current {
                // Found a newline; Return all whitespace found up to this point.
                // TODO: merge consecutive WS
                '\n' => {
                    self.inc(); // TODO: eof
                    return Ok(Some((None, Item::WS(self.extract()))));
                }
                // Skip whitespace.
                ' ' | '\t' | '\r' => {
                    if !self.inc() {
                        return Ok(Some((None, Item::WS(self.extract()))));
                    }
                }
                // Found a comment, parse it.
                '#' => {
                    let indent = self.extract();
                    let (cws, comment, trail) = self.parse_comment_trail();
                    return Ok(Some((
                        None,
                        Item::Comment(Trivia {
                            indent: indent,
                            comment_ws: cws,
                            comment: comment,
                            trail: trail,
                        }),
                    )));
                }
                // Found a table, delegate to the calling function.
                '[' => return Ok(None),
                // Begining of a KV pair.
                // Return to beginning of whitespace so it gets included
                // as indentation for the KV about to be parsed.
                _ => {
                    self.idx = self.marker;
                    let (key, value) = self.parse_key_value(true)?;
                    return Ok(Some((key, value)));
                }
            }
        }
    }

    /// Returns `(comment_ws, comment, trail)`
    /// If there is no comment, comment_ws and comment will
    /// simply be empty.
    pub fn parse_comment_trail(&mut self) -> (&'a str, &'a str, &'a str) {
        if self.end() {
            return ("", "", "");
        }
        
        let mut comment = "";
        let mut comment_ws = "";
        self.mark();
        
        loop {
            match self.current {
                '\n' => break,
                '#' => {
                    comment_ws = self.extract();
                    self.mark();
                    self.inc(); // Skip #

                    // The comment itself
                    while !self.end() && !self.current.is_nl() && self.inc() {}
                    comment = self.extract();
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
        while self.inc() && self.current.is_spaces() {}
        if self.current == '\r' {
            self.inc();
        }
        if self.current == '\n' {
            self.inc();
        }

        let trail = if self.idx != self.marker || self.current.is_ws() {
            self.extract()
        } else {
            ""
        };
        (comment_ws, comment, trail)
    }

    /// Parses and returns a key/value pair.
    pub fn parse_key_value(&mut self, parse_comment: bool) -> Result<(Option<Key<'a>>, Item<'a>)> {
        // Leading indent.
        self.mark();
        while self.current.is_spaces() && self.inc() {}
        let indent = self.extract();

        // Key.
        let mut key = self.parse_key();
        self.mark();
        while self.current.is_kv_sep() && self.inc() {}
        key.sep = self.extract_exact();

        // Value.
        let mut val = self.parse_val()?;

        // Comment
        if parse_comment {
            let (cws, comment, trail) = self.parse_comment_trail();
            let meta = val.meta_mut();
            meta.comment_ws = cws;
            meta.comment = comment;
            meta.trail = trail;
        }
        val.meta_mut().indent = indent;

        Ok((Some(key), val))
    }

    /// Attempts to parse a value at the current position.
    pub fn parse_val(&mut self) -> Result<Item<'a>> {
        self.mark();
        let trivia: Trivia = Default::default();
        match self.current {
            '"' => self.parse_basic_string(),
            '\'' => self.parse_literal_string(),
            // Boolean: true
            't' if &self.src[self.idx..self.idx + 4] == "true" => {
                self.inc();
                self.inc();
                self.inc();
                self.inc();

                Ok(Item::Bool {
                    val: true,
                    meta: trivia,
                })
            }
            // Boolean: False
            'f' if &self.src[self.idx..self.idx + 5] == "false" => {
                self.inc();
                self.inc();
                self.inc();
                self.inc();
                self.inc();

                Ok(Item::Bool {
                    val: false,
                    meta: trivia,
                })
            }
            // Array
            '[' => {
                let mut elems: Vec<Item> = Vec::new();
                self.inc();

                while self.current != ']' {
                    self.mark();
                    while self.current.is_ws() || self.current == ',' {
                        self.inc();
                    }
                    if self.idx != self.marker {
                        elems.push(Item::WS(self.extract_exact()));
                    }
                    if self.current == ']' {
                        break;
                    }
                    let next = match self.current {
                        '#' => {
                            let (cws, comment, trail) = self.parse_comment_trail();
                            Item::Comment(Trivia {
                                indent: "",
                                comment_ws: cws,
                                comment: comment,
                                trail: trail,
                            })
                        }
                        _ => self.parse_val()?,
                    };
                    elems.push(next);
                }
                self.inc();

                let res = Item::Array {
                    val: elems,
                    meta: trivia,
                };

                if res.is_homogeneous() {
                    Ok(res)
                } else {
                    bail!(ErrorKind::MixedArrayTypes);
                }
            }
            // Inline Table
            '{' => {
                let mut elems: Container = Container::new();
                self.inc();

                while self.current != '}' {
                    while self.current.is_ws() || self.current == ',' {
                        self.inc();
                    }
                    let (key, val) = self.parse_key_value(false)?;
                    let _ = elems.append(key, val)?;
                }
                self.inc();

                Ok(Item::InlineTable {
                    val: elems,
                    meta: trivia,
                })
            }
            // Integer, Float, or DateTime
            '+' | '-' | '0'...'9' => {
                while self.current.not_in(" \t\n\r#,]}") && self.inc() {}

                // TODO: EOF shittiness
                // if !self.current.is_digit(10) {
                //     self.idx -= 1;
                // }

                let raw = self.extract();

                let clean: String = raw.chars()
                    .filter(|c| *c != '_' && *c != ' ')
                    .collect::<String>();

                // Forgiveness > Permission
                if let Ok(res) = i64::from_str(&clean) {
                    return Ok(Item::Integer {
                        val: res,
                        meta: trivia,
                        raw: raw,
                    });
                } else if let Ok(res) = f64::from_str(&clean) {
                    // TODO: "Similar to integers, you may use underscores to enhance
                    // readability. Each underscore must be surrounded by at least one digit."
                    return Ok(Item::Float {
                        val: res,
                        meta: trivia,
                        raw: raw,
                    });
                } else if let Ok(res) = ChronoDateTime::parse_from_rfc3339(&clean) {
                    return Ok(Item::DateTime {
                        val: res,
                        raw: raw, // XXX this was `clean`, why?
                        meta: trivia,
                    });
                } else {
                    bail!(ErrorKind::InvalidNumberOrDate);
                }
            }
            ch => bail!(ErrorKind::UnexpectedChar(ch)),
        }
    }

    fn parse_literal_string(&mut self) -> Result<Item<'a>> {
        self.parse_string('\'')
    }

    fn parse_basic_string(&mut self) -> Result<Item<'a>> {
        self.parse_string('"')
    }

    fn parse_string(&mut self, delim: char) -> Result<Item<'a>> {
        // TODO: Handle escaping.
        let mut multiline = false;
        let mut str_type = if delim == '\'' {
            StringType::SLL
        } else {
            StringType::SLB
        };
        // Skip opening delim.
        self.inc() || bail!(ErrorKind::UnexpectedEof);
        if self.current == delim {
            self.inc();
            if self.current == delim {
                multiline = true;
                str_type = if delim == '\'' {
                    StringType::MLL
                } else {
                    StringType::MLB
                };
                self.inc() || bail!(ErrorKind::UnexpectedEof);
            } else {
                // Empty string.
                return Ok(Item::Str {
                    t: str_type,
                    val: "",
                    original: "",
                    meta: Default::default(),
                });
            }
        }
        self.mark();

        'outer: loop {
            if self.current == delim {
                let val = self.extract_exact();
                if multiline {
                    for _ in 0..3 {
                        if self.current != delim {
                            // Not a triple quote, leave in result as-is.
                            continue 'outer;
                        }
                        self.inc(); // TODO: Handle EOF
                    }
                } else {
                    self.inc();
                }
                return Ok(Item::Str {
                    t: str_type,
                    val: val,
                    original: val,
                    meta: Default::default(),
                });
            } else {
                self.inc() || bail!(ErrorKind::UnexpectedEof);
            }
        }
    }

    /// Parses a Key at the current position;
    /// WS before the key must be exhausted first at the callsite.
    fn parse_key(&mut self) -> Key<'a> {
        let key = match self.current {
            '"' | '\'' => self.parse_quoted_key(),
            _ => self.parse_bare_key(),
        };
        key
    }

    /// Parses a key enclosed in either single or double quotes.
    fn parse_quoted_key(&mut self) -> Key<'a> {
        let quote_style = self.current;
        let key_type = match quote_style {
            '"' => KeyType::Basic,
            '\'' => KeyType::Literal,
            _ => panic!("Should not have entered parse_quoted_key()"),
        };
        self.inc();
        self.mark();

        while self.current != quote_style && self.inc() {}
        let key = self.extract();
        self.inc();

        Key {
            t: key_type,
            sep: "",
            key: key,
        }
    }

    /// Parses a bare key
    fn parse_bare_key(&mut self) -> Key<'a> {
        self.mark();
        while self.current.is_bare_key_char() && self.inc() {}
        let key = self.extract();

        Key {
            t: KeyType::Bare,
            sep: "",
            key: key,
        }
    }

    /// Peeks ahead non-intrusively by cloning then restoring the
    /// initial state of the parser.
    /// Returns the name of the table about to be parsed,
    /// as well as whether it is part of an AoT.
    fn peek_table(&mut self) -> Result<(isAOT, &'a str)> {
        // Save initial state
        let chars = self.chars.clone();
        let idx = self.idx;
        let current = self.current;
        let marker = self.marker;

        // FIXME: May need changing to allow leading indentation
        if self.current != '[' {
            bail!(ErrorKind::InternalParserError(
                "Peek_table_name entered on non-bracket character".into(),
            ));
        }

        // AoT?
        self.inc();
        let is_AOT = match self.current {
            '[' => {
                println!("AOT");
                self.inc();
                true as isAOT
            }
            _ => false,
        };

        self.mark();
        while self.inc() && self.current != ']' {}
        let table_name = self.extract();

        // Restore initial state
        self.chars = chars;
        self.idx = idx;
        self.current = current;
        self.marker = marker;

        Ok((is_AOT, table_name))
    }

    /// Parse a table element.
    pub fn parse_table(&mut self) -> Result<(Key<'a>, Item<'a>)> {
        let indent = self.extract();
        self.inc(); // Skip opening bracket.

        let is_aot = if self.current == '[' {
            self.inc() || bail!(ErrorKind::UnexpectedEof)
        } else {
            false
        };

        // Key
        self.mark();
        // TODO:, handle EOF.
        while self.current != ']' && self.inc() {}

        // TODO: Key parsing and validation.
        let name = self.extract_exact();
        let key = Key {
            t: KeyType::Bare,
            sep: "",
            key: name.clone(),
        };
        self.inc(); // Skip closing bracket.
        if is_aot {
            // TODO: Verify close bracket.
            self.inc();
        }
        // --------------------------

        let (cws, comment, trail) = self.parse_comment_trail();

        // CLEANUP: Total hack, add undecided variant
        let mut result = Item::integer("999");
        let mut values = Container::new();
        // CLEANUP: Wait for table API:
        // Use table API to add kv's as they come so result is never
        // uninitialized
        while !self.end() {
            if let Some((key, item)) = self.parse_item()? {
                values.append(key, item)?;
            } else {
                if self.current == '[' {
                    let (_, name_next) = self.peek_table()?;

                    if Parser::is_child(name, name_next) {
                        let (key_next, table_next) = self.parse_table()?;
                        values.append(key_next, table_next)?;
                    } else {
                        let table = Item::Table {
                            is_array: is_aot,
                            val: values.clone(),
                            meta: Trivia {
                                indent: indent,
                                comment_ws: cws,
                                comment: comment,
                                trail: trail,
                            },
                        };
                        result = if is_aot && (self.AoT_stack.is_empty() || name != *self.AoT_stack.last().unwrap()) {
                            self.parse_aot(table, name)?
                        } else {
                            table
                        }
                    }
                    break;
                } else {
                    bail!(ErrorKind::InternalParserError(
                        "parse_item() returned None on a non-bracket character."
                            .into(),
                    ))
                }
            }
        }
        // CLEANUP: undecided variant
        if result.is_integer() {
            result = Item::Table {
                is_array: is_aot,
                val: values.clone(),
                meta: Trivia {
                    indent: indent,
                    comment_ws: cws,
                    comment: comment,
                    trail: trail,
                },
            };
        }
        Ok((key, result))
    }

    fn parse_aot(&mut self, first: Item<'a>, name_first: &'a str) -> Result<Item<'a>> {
        // We are in an AoT, and next is not a child of first.
        let mut payload = vec![first];
        self.AoT_stack.push(name_first);
        while !self.end() {
            let (is_aot_next, name_next) = self.peek_table()?;
            if is_aot_next && name_next == name_first {
                let (_, table) = self.parse_table()?;
                payload.push(table);
            } else {
                break;
            }
        }
        self.AoT_stack.pop();
        Ok(Item::AoT(payload))
    }
}
