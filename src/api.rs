use items::*;
use container::*;
use errors::*;

use chrono::DateTime as ChronoDateTime;

// @todo: passing reference because cow not implemented.

pub fn integer(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::Integer {
        val: raw.parse::<i64>()?,
        trivia: Trivia::new(),
        raw: raw,
    })
}

pub fn float(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::Float {
        val: raw.parse::<f64>()?,
        trivia: Trivia::new(),
        raw: raw,
    })
}

pub fn bool(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::Bool {
        val: raw.parse::<bool>()?,
        trivia: Trivia::new(),
    })
}

pub fn datetime(raw: &'static str) -> Result<Item<'static>> {
    Ok(Item::DateTime {
        val: ChronoDateTime::parse_from_rfc3339(raw)?,
        trivia: Trivia::new(),
        raw: raw,
    })
}

pub fn array<'a>() -> Result<Item<'a>> {
    Ok(Item::Array {
        // @todo: Average length of toml arrays?
        val: Vec::with_capacity(10),
        trivia: Trivia::new(),
    })
}

pub fn table<'a>() -> Item<'a> {
    Item::Table {
        is_aot_elem: false,
        val: Container::new(),
        trivia: Trivia::new(),
    }
}

pub fn inline_table<'a>() -> Item<'a> {
    Item::InlineTable {
        val: Container::new(),
        trivia: Trivia::new(),
    }
}

pub fn aot<'a>() -> Item<'a> {
    Item::AoT(Vec::with_capacity(5))
}

pub fn value<'a>(src: &'a str) -> Result<Item<'a>> {
    let mut parser = ::parser::Parser::new(src);
    parser.parse_value()
}

pub fn key_value<'a>(src: &'a str) -> Result<(Key<'a>, Item<'a>)> {
    let mut parser = ::parser::Parser::new(src);
    parser.parse_key_value(true)
}

// @cleanup: How should the string be passed?
// - With delimiters? Ugly and requires raw strings or escaping
// - Without delimiters? Nicer but requires inspecting the string to
// determine its type.
// Either way this puts "soft" requirements on the API that I don't like,
// but how to typesystem this instead?
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
    pub fn append<K: Into<Option<Key<'a>>>>(&mut self, _key: K, item: Item<'a>) -> Result<()> {
        use Item::*;
        match *self {
            Table { ref mut val, .. } |
            InlineTable { ref mut val, .. } => val.append(_key, item),
            Array { .. } | AoT { .. } => unimplemented!(),
            _ => bail!(ErrorKind::APIWrongItem),
        }
    }

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
    pub fn is_date_time(&self) -> bool {
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
