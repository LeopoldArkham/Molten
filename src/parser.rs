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
    /// Creates a new parser from a &str.
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

    /// Extracts the value between marker and index.
    fn extract(&mut self) -> &'a str {
        if self.end() {
            &self.src[self.marker..]
        } else {
            &self.src[self.marker..self.idx]
        }
    }

    // HACK: this should go away
    /// Extracts the exact value between marker and index.
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

    /// Saves the current position and all related information, to be passed to
    /// `restore_idx` later if the parser needs to rewind for any reason. Consistently
    /// using these methods will make sure the cursor stays consistent.
    fn save_idx(&self) -> (CharIndices<'a>, usize, char) {
        (self.chars.clone(), self.idx, self.current)
    }

    /// Restores the position that was saved with `save_idx`.
    fn restore_idx(&mut self, (chars, idx, current): (CharIndices<'a>, usize, char)) {
        self.chars = chars;
        self.idx = idx;
        self.current = current;
    }

    /// Converts a byte offset from an error message to a (line, column) pair.
    ///
    /// All indexes are 0-based.
    fn to_linecol(&self, offset: usize) -> (usize, usize) {
        let mut cur = 0;
        for (i, line) in self.src.lines().enumerate() {
            if cur + line.len() + 1 > offset {
                return (i, offset - cur);
            }
            cur += line.len() + 1;
        }
        (self.src.lines().count(), 0)
    }

    /// Create a generic "parse error" at the current position.
    fn parse_error(&self) -> Error {
        let (line, col) = self.to_linecol(self.idx);

        ErrorKind::ParseError(line, col).into()
    }

    /// Create an error with the given kind wrapped in a "parse error"
    /// containing the current position.
    fn error(&self, err: ErrorKind) -> Error {
        Error::from_kind(err).chain_err(|| self.parse_error())
    }

    /// Merges the given `Item` with the last one currently in the given `Container` if
    /// both are whitespace items. Returns `true` if the items were merged.
    fn merge_ws<'b>(&self, item: &'b Item<'a>, container: &'b mut Container<'a>) -> bool {
        if let Some(last) = container.last_item_mut() {
            if let (&&mut Item::WS(prefix), &Item::WS(suffix)) = (&last, item) {
                let start = self.idx - (prefix.len() + suffix.len());
                *last = Item::WS(&self.src[start..self.idx]);
                return true;
            }
        }
        false
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
                if !self.merge_ws(&value, &mut body) {
                    body.append(key, value).chain_err(|| self.parse_error())?;
                }

                self.mark();
            } else {
                break;
            }
        }

        // Switch to parsing tables/arrays of tables until the end of the input.
        while !self.end() {
            let (k, v) = self.parse_table()?;
            let v = match v {
                Item::Table { is_aot_elem, .. } if is_aot_elem => {
                    // This is just the first table in an AoT. Parse the rest of the array
                    // along with it.
                    self.parse_aot(v, k.key)?
                }
                _ => v,
            };
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
        let saved_idx = self.save_idx();
        loop {
            match self.current {
                // Found a newline; Return all whitespace found up to this point.
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
                    self.restore_idx(saved_idx);
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
        while self.current.is_spaces() && self.inc() {}
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
            't' if self.src[self.idx..].starts_with("true") => {
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
            'f' if self.src[self.idx..].starts_with("false") => {
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
                    Err(self.error(ErrorKind::MixedArrayTypes))
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

                let raw = self.extract();

                if let Some(item) = Parser::parse_number(raw, trivia.clone()) {
                    return Ok(item);
                } else if let Ok(res) = ChronoDateTime::parse_from_rfc3339(raw) {
                    return Ok(Item::DateTime {
                        val: res,
                        raw: raw, // XXX this was `clean`, why?
                        meta: trivia,
                    });
                } else {
                    Err(self.error(ErrorKind::InvalidNumberOrDate))
                }
            }
            ch => Err(self.error(ErrorKind::UnexpectedChar(ch))),
        }
    }

    fn parse_number(raw: &'a str, trivia: Trivia<'a>) -> Option<Item<'a>> {
        // Leading zeros are not allowed
        if raw.len() > 1 && raw.starts_with('0') && !raw.starts_with("0.") {
            return None;
        }

        // Underscores should be surrounded by digits
        let (valid, last) = raw.chars().fold((true, None), |(valid, prev): (bool,
                         Option<char>),
         c: char| {
            if !valid {
                return (false, None);
            }
            (
                match (prev, c) {
                    (None, '_') => false,
                    (Some(x), '_') | (Some('_'), x) => x.is_digit(10),
                    _ => true,
                },
                Some(c),
            )
        });

        if !valid || last == Some('_') {
            return None;
        }

        // And it should parses as an int or a float
        let clean: String = raw.chars()
            .filter(|c| *c != '_' && *c != ' ')
            .collect::<String>();

        // Forgiveness > Permission
        if let Ok(res) = i64::from_str(&clean) {
            return Some(Item::Integer {
                val: res,
                meta: trivia,
                raw,
            });
        } else if let Ok(res) = f64::from_str(&clean) {
            return Some(Item::Float {
                val: res,
                meta: trivia,
                raw,
            });
        }

        None
    }

    fn parse_literal_string(&mut self) -> Result<Item<'a>> {
        self.parse_string('\'')
    }

    fn parse_basic_string(&mut self) -> Result<Item<'a>> {
        self.parse_string('"')
    }

    /// Parses a string element
    fn parse_string(&mut self, delim: char) -> Result<Item<'a>> {
        // TODO: Handle escaping.
        let mut multiline = false;
        let mut str_type = if delim == '\'' {
            StringType::SLL
        } else {
            StringType::SLB
        };
        // Skip opening delim.
        self.inc() || return Err(self.error(ErrorKind::UnexpectedEof));
        if self.current == delim {
            self.inc();
            if self.current == delim {
                multiline = true;
                str_type = if delim == '\'' {
                    StringType::MLL
                } else {
                    StringType::MLB
                };
                self.inc() || return Err(self.error(ErrorKind::UnexpectedEof));
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
                self.inc() || return Err(self.error(ErrorKind::UnexpectedEof));
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
        let idx = self.save_idx();
        let marker = self.marker;

        // FIXME: May need changing to allow leading indentation
        if self.current != '[' {
            return Err(self.error(ErrorKind::InternalParserError(
                "Peek_table entered on non-bracket character".into(),
            )));
        }

        // AoT?
        self.inc();
        let is_AOT = match self.current {
            '[' => {
                self.inc();
                true as isAOT
            }
            _ => false,
        };

        self.mark();
        while self.inc() && self.current != ']' {}
        let table_name = self.extract();

        // Restore initial state
        self.restore_idx(idx);
        self.marker = marker;

        Ok((is_AOT, table_name))
    }

    /// Parses a table element.
    pub fn parse_table(&mut self) -> Result<(Key<'a>, Item<'a>)> {
        let indent = self.extract();
        self.inc(); // Skip opening bracket.

        let is_aot = if self.current == '[' {
            self.inc() || return Err(self.error(ErrorKind::UnexpectedEof))
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
        let mut result = Item::None;
        let mut values = Container::new();
        while !self.end() {
            if let Some((key, item)) = self.parse_item()? {
                if !self.merge_ws(&item, &mut values) {
                    values.append(key, item)?;
                }
            } else {
                if self.current == '[' {
                    let (_, name_next) = self.peek_table()?;

                    if Parser::is_child(name, name_next) {
                        let (key_next, table_next) = self.parse_table()?;
                        values.append(key_next, table_next)?;
                    } else {
                        let table = Item::Table {
                            is_aot_elem: is_aot,
                            val: values.clone(),
                            meta: Trivia {
                                indent: indent,
                                comment_ws: cws,
                                comment: comment,
                                trail: trail,
                            },
                        };
                        result = if is_aot &&
                            (self.AoT_stack.is_empty() || name != *self.AoT_stack.last().unwrap())
                        {
                            self.parse_aot(table, name)?
                        } else {
                            table
                        }
                    }
                    break;
                } else {
                    return Err(self.error(ErrorKind::InternalParserError(
                        "parse_item() returned None on a non-bracket character.".into(),
                    )));
                }
            }
        }
        // CLEANUP: undecided variant
        if result.is_none() {
            result = Item::Table {
                is_aot_elem: is_aot,
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

    /// Parses all siblings of the provided table `first` and
    /// bundles them into an AoT.
    fn parse_aot(&mut self, first: Item<'a>, name_first: &'a str) -> Result<Item<'a>> {
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

#[cfg(test)]
mod tests {

    use parser::*;

    #[test]
    fn invalid_numbers() {
        let invalid_ints = vec!["01", "_1", "1_", "1__2"];
        let invalid_floats = vec!["00.1", "_1.0", "1.0_", "1_.0"];
        invalid_ints.iter().for_each(|s| {
            assert_eq!(None, Parser::parse_number(s, Trivia::default()))
        });
        invalid_floats.iter().for_each(|s| {
            assert_eq!(None, Parser::parse_number(s, Trivia::default()))
        });
    }

    #[test]
    fn issue41() {
        let text = ::std::str::from_utf8(b"\'\'fb\'\xee\x9d\xbd").unwrap();
        let _ = Parser::new(text).parse();
    }

    #[test]
    fn issue42() {
        let text = ::std::str::from_utf8(b"\'\nv\'f%\nb").unwrap();
        let _ = Parser::new(text).parse();
    }
}
