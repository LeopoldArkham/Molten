//! Molten Public API

use items::*;
use container::*;
use errors::*;

use chrono::DateTime as ChronoDateTime;

// TODO: passing reference because cow not implemented.

/// Return an integer `Item` parsed from the text`str`.
pub fn integer(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::Integer {
        val: raw.parse::<i64>()?,
        trivia: Trivia::new(),
        raw: raw,
    })
}

/// Return a float `Item` parsed from the text `str`.
pub fn float(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::Float {
        val: raw.parse::<f64>()?,
        trivia: Trivia::new(),
        raw: raw,
    })
}

/// Return a bool `Item` parsed from the text `str`.
pub fn bool(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::Bool {
        val: raw.parse::<bool>()?,
        trivia: Trivia::new(),
    })
}

/// Return a datetime `Item` parsed from the text `str`.
pub fn datetime(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::DateTime {
        val: ChronoDateTime::parse_from_rfc3339(raw)?,
        trivia: Trivia::new(),
        raw: raw,
    })
}

/// Return a array `Item` parsed from the text `str`.
pub fn array<'a>() -> Result<Item<'a>> {
    Ok(Item::Array {
        // TODO: Average length of toml arrays?
        val: Vec::with_capacity(10),
        trivia: Trivia::new(),
    })
}

/// Return a table `Item` parsed from the text `str`.
pub fn table<'a>() -> Result<Item<'a>> {
    Ok(Item::Table {
        is_aot_elem: false,
        val: Container::new(),
        trivia: Trivia::new(),
    })
}

/// Return an inline table `Item` parsed from the text `str`.
pub fn inline_table<'a>() -> Result<Item<'a>> {
    Ok(Item::InlineTable {
        val: Container::new(),
        trivia: Trivia::new(),
    })
}

/// Return an aot `Item` parsed from the text `str`.
pub fn aot<'a>() -> Result<Item<'a>> {
    Ok(Item::AoT(Vec::with_capacity(5)))
}

/// Return a value `Item` parsed from the text `str`.
pub fn value<'a>(src: &'a str) -> Result<Item<'a>> {
    let mut parser = ::parser::Parser::new(src);
    parser.parse_value()
}

/// Return a key-value `Item` parsed from the text `str`.
pub fn key_value<'a>(src: &'a str) -> Result<(Key<'a>, Item<'a>)> {
    let mut parser = ::parser::Parser::new(src);
    parser.parse_key_value(true)
}

// TODO: How should the string be passed?
// - With delimiters? Ugly and requires raw strings or escaping
// - Without delimiters? Nicer but requires inspecting the string to
//   determine its type.
//
// Either way this puts "soft" requirements on the API that I don't like. Can
// the type system be used to enforce these instead?

/// Return a string `Item` parsed from the text `str`.
pub fn string<'a>(raw: &'a str) -> Result<Item<'a>> {
    let mut parser = ::parser::Parser::new(raw.as_ref());
    let value = parser.parse_value()?;

    if !value.is_string() {
        bail!(ErrorKind::ParseStringError);
    }

    Ok(value)
}

/// Append - Remove
impl<'a> Item<'a> {
    /// Append a (key, value) to the current table.
    pub fn append<K: Into<Option<Key<'a>>>>(&mut self, key: K, item: Item<'a>) -> Result<()> {
        use Item::*;
        match *self {
            Table { ref mut val, .. } |
            InlineTable { ref mut val, .. } => val.append(key, item),
            Array { .. } | AoT { .. } => unimplemented!(),
            _ => bail!(ErrorKind::APIWrongItem),
        }
    }

    /// Remove the (key, value) `key` from the current table.
    pub fn remove(&mut self, key: &Key<'a>) -> Result<()> {
        use Item::*;
        match *self {
            Table { ref mut val, .. } |
            InlineTable { ref mut val, .. } => val.remove(key),
            Array { .. } | AoT { .. } => unimplemented!(),
            _ => bail!(ErrorKind::APIWrongItem),
        }
    }
}

/// Identity methods
impl<'a> Item<'a> {
    /// Returns true if Item is a value.
    pub fn is_value(&self) -> bool {
        match self.discriminant() {
            0 | 1 => false,
            _ => true,
        }
    }

    /// Returns true if Item is trivia.
    pub fn is_trivia(&self) -> bool {
        !self.is_value()
    }

    /// Returns true if Item is whitespace.
    pub fn is_ws(&self) -> bool {
        self.discriminant() == 0
    }

    /// Returns true if Item is a comment.
    pub fn is_comment(&self) -> bool {
        self.discriminant() == 1
    }

    /// Returns true if Item is an integer.
    pub fn is_integer(&self) -> bool {
        self.discriminant() == 2
    }

    /// Returns true if Item is a float.
    pub fn is_float(&self) -> bool {
        self.discriminant() == 3
    }

    /// Returns true if Item is a boolean.
    pub fn is_bool(&self) -> bool {
        self.discriminant() == 4
    }

    /// Returns true if Item is a date/time.
    pub fn is_datetime(&self) -> bool {
        self.discriminant() == 5
    }

    /// Returns true if Item is an array.
    pub fn is_array(&self) -> bool {
        self.discriminant() == 6
    }

    /// Returns true if Item is a table.
    pub fn is_table(&self) -> bool {
        self.discriminant() == 7 || self.discriminant() == 8
    }

    /// Returns true if Item is an inline table.
    pub fn is_inline_table(&self) -> bool {
        self.discriminant() == 8
    }

    /// Returns true if Item is a string.
    pub fn is_string(&self) -> bool {
        self.discriminant() == 9
    }

    /// Returns true if Item is AOT.
    pub fn is_aot(&self) -> bool {
        self.discriminant() == 10
    }

    /// Returns true if Item is None.
    pub fn is_none(&self) -> bool {
        self.discriminant() == 11
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_integer() {
        // tests contains tuples of a string to test and a boolean that
        // indicates whether the result of parsing that string should be valid
        // (true), or not (false).
        let tests = vec![("99572", true), ("37.2", false)];
        for (v, r) in tests {
            let i = integer(v);
            if r {
                assert!(i.is_ok());
                assert!(i.unwrap().is_integer());
            } else {
                assert!(i.is_err());
            }
        }
    }

    #[test]
    fn api_float() {
        // tests contains tuples of a string to test and a boolean that
        // indicates whether the result of parsing that string should be valid
        // (true), or not (false).
        let tests = vec![("39581.102", true), ("12577.2", true), ("385", true)];
        for (v, r) in tests {
            let i = float(v);
            if r {
                assert!(i.is_ok());
                assert!(i.unwrap().is_float());
            } else {
                assert!(i.is_err());
            }
        }
    }

    #[test]
    fn api_bool() {
        // tests contains tuples of a string to test and a boolean that
        // indicates whether the result of parsing that string should be valid
        // (true), or not (false).
        let tests = vec![("true", true), ("false", true), ("blarg", false)];
        for (v, r) in tests {
            let i = bool(v);
            if r {
                assert!(i.is_ok());
                assert!(i.unwrap().is_bool());
            } else {
                assert!(i.is_err());
            }
        }
    }

    #[test]
    fn api_string() {
        // tests contains tuples of a string to test and a boolean that
        // indicates whether the result of parsing that string should be valid
        // (true), or not (false).
        let tests = vec![
            ("'my string'", true),
            ("\"My string\"", true),
            ("'1234'", true),
        ];

        for (v, r) in tests {
            let i = string(v);
            if r {
                assert!(i.is_ok());
                assert!(i.unwrap().is_string());
            } else {
                assert!(i.is_err());
            }
        }
    }

    #[test]
    #[ignore]
    /// Datetimes are [RFC-3339](https://tools.ietf.org/html/rfc3339)-compliant
    /// date or time strings.

    // BUG(markcol): fix parsing problems with date times. According to the
    // [TOML spec](https://github.com/toml-lang/toml#user-content-local-date),
    // the tests below should work, but fail.
    fn api_datetime() {
        // tests contains tuples of a string to test and a boolean that
        // indicates whether the result of parsing that string should be valid
        // (true), or not (false).
        let tests = vec![("1979-05-20", true), ("1974-5-20T11:05Z", true)];
        for (v, r) in tests {
            let i = datetime(v);
            if r {
                assert!(i.is_ok());
                assert!(i.unwrap().is_datetime());
            } else {
                assert!(i.is_err());
            }
        }
    }

    #[test]
    fn api_array() {
        let i = array();
        assert!(i.is_ok());
        assert!(i.unwrap().is_array());

        // TODO(markcol): add tests for append/remove
    }

    #[test]
    fn api_table() {
        let i = table();
        assert!(i.is_ok());
        assert!(i.unwrap().is_table());

        // TODO(markcol): add tests for append/remove
        // assert!(i.append("key1", string("\"some string\"").unwrap()).is_ok());
        // assert!(i.append("key2", integer("123").unwrap()).is_ok());
    }

    #[test]
    fn api_inline_table() {
        let i = inline_table();
        assert!(i.is_ok());
        assert!(i.unwrap().is_inline_table());

        // TODO(markcol): add tests for append/remove
    }

    #[test]
    #[ignore]
    fn api_is_trivia() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn api_is_value() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn api_is_ws() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn api_is_comment() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn api_is_none() {
        unimplemented!();
    }
}