//! Items are the basic elements of a `TOMLDocument`.

use chrono::{DateTime as ChronoDateTime, FixedOffset};
use errors::*;
use container::Container;

use std::borrow::Cow;
use std::fmt;

/// Type of TOML string.
///
/// There are four ways to express strings: basic, multi-line basic,
/// literal, and multi-line literal. All strings must contain only valid UTF-8
/// characters.
///
/// **Basic strings** are surrounded by quotation marks. Any Unicode
/// character may be used except those that must be escaped: quotation mark,
/// backslash, and the control characters (U+0000 to U+001F).
///
/// ```text
/// str = "I'm a string. \"You can quote me\". Name\tJos\u00E9\nLocation\tSF."
/// ```
///
/// For convenience, common characters have a compact escape sequence.
///
/// | Escape       | Name            | Unicode Replacement |
/// |--------------|-----------------|---------------------|
/// | \b           | backspace       | (U+0008)            |
/// | \t           | tab             | (U+0009)            |
/// | \n           | linefeed        | (U+000A)            |
/// | \f           | form feed       | (U+000C)            |
/// | \r           | carriage return | (U+000D)            |
/// | \"           | quote           | (U+0022)            |
/// | \\           | backslash       | (U+005C)            |
/// | `\uXXXX`     | unicode         | (U+XXXX)            |
/// | `\UXXXXXXXX` | unicode         | (U+XXXXXXXX)        |
///
/// Any Unicode character may be escaped with the `\uXXXX` or `\UXXXXXXXX`
/// forms. The escape codes must be valid Unicode scalar values.
///
/// All other escape sequences not listed above are reserved and, if used,
/// should produce an error.
///
/// **Multi-line basic** strings are surrounded by three quotation marks on
/// each side and allow newlines. A newline immediately following the opening
/// delimiter will be trimmed. All other whitespace and newline characters
/// remain intact.
///
/// **Literal strings** are surrounded by single quotes. Like basic strings,
/// they must appear on a single line. Literal strings do not allow escaping
/// of characters within the string.
///
/// **Multi-line literal** strings are surrounded by three single-quotes on
/// each side and allow newlines. A newline immediately following the opening
/// delimiter will be trimmed. All other whitespace and newline characters
/// remain intact. No escaping is allowed within the string.
#[derive(Debug, Clone, PartialEq)]
pub enum StringType {
    /// Single line basic string.
    SLB,
    /// Multi-line basic string.
    MLB,
    /// Single-line literal string.
    SLL,
    /// Multi-line literal string.
    MLL,
}

impl StringType {
    /// Return the delimiter applicable for the given `StringType'.
    pub fn delimiter(&self) -> &'static str {
        match *self {
            StringType::SLB => "\"",
            StringType::MLB => "\"\"\"",
            StringType::SLL => "'",
            StringType::MLL => "'''",
        }
    }
}

/// Trivia information (aka metadata).
#[derive(Default, Clone, PartialEq)]
pub struct Trivia<'a> {
    /// Whitespace before a value.
    pub indent: &'a str,
    /// Whitespace after a value, but before a comment.
    pub comment_ws: &'a str,
    /// Comment, starting with # character, or empty string if no comment.
    pub comment: &'a str,
    /// Trailing newline.
    pub trail: &'a str,
}

impl<'a> Trivia<'a> {
    /// Creates an empty Trivia with OS-specific newline.
    pub fn new() -> Trivia<'a> {
        Trivia {
            indent: "",
            comment_ws: "",
            comment: "",
            trail: ::NL,
        }
    }
}

impl<'a> fmt::Debug for Trivia<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

/// The type of a `Key`.
/// Keys can be bare (unquoted), or quoted using basic ("), or literal (')
/// quotes following the same escaping rules as single-line `StringType`.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum KeyType {
    /// Bare key.
    Bare,
    /// Basic key.
    Basic,
    /// Literal key.
    Literal,
}

/// A key value.
#[derive(Clone)]
pub struct Key<'a> {
    /// The type of the key
    pub t: KeyType,
    /// The key separator
    pub sep: &'a str,
    /// The actual key value
    pub key: Cow<'a, str>,
    /// The raw key
    pub raw: Cow<'a, str>,
}

impl<'a> Key<'a> {
    /// Creates a new bare key with a standard separator
    pub fn new<K: Into<Cow<'a, str>> + Clone>(k: K) -> Key<'a> {
        let k = k.into();
        let key = Cow::Owned(k.trim().to_string());
        Key {
            t: KeyType::Bare,
            sep: " = ",
            key: key,
            raw: k.into(),
        }
    }

    /// Creates a new bare key with a standard separator
    pub fn new_with_type<K: Into<Cow<'a, str>> + Clone>(k: K, key_type: KeyType) -> Key<'a> {
        let k = k.into();
        let key = Cow::Owned(k.trim().to_string());
        Key {
            t: key_type,
            sep: " = ",
            key: key,
            raw: k.into(),
        }
    }

    /// Returns the delimiter used by the given `KeyType'.
    pub fn delimiter(&self) -> &'static str {
        match self.t {
            KeyType::Bare => "",
            KeyType::Basic => "\"",
            KeyType::Literal => "'",
        }
    }
}

impl<'a> Eq for Key<'a> {}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, other: &Key) -> bool {
        self.key == other.key
    }
}

impl<'a> ::std::hash::Hash for Key<'a> {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<'a> ::std::fmt::Debug for Key<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}{}{}", self.delimiter(), self.key, self.delimiter())
    }
}

impl<'a> Key<'a> {
    /// Returns the string represenation of a `Key`.
    pub fn as_string(&self) -> String {
        format!("{}{}{}", self.delimiter(), self.raw, self.delimiter())
    }
}

/// An item within a TOML document.
#[derive(Debug, Clone, PartialEq)]
pub enum Item<'a> {
    /// A whitespace literal.
    WS(&'a str),
    /// A comment literal.
    Comment(Trivia<'a>),
    /// An integer literal.
    Integer {
        /// The value of the integer.
        val: i64,
        /// Trivia for the integer.
        trivia: Trivia<'a>,
        /// The original representation of the integer value.
        raw: &'a str,
    },
    /// A float literal.
    Float {
        /// The value of the float.
        val: f64,
        /// Trivia data for the Float.
        trivia: Trivia<'a>,
        /// The original string representation of the value.
        raw: &'a str,
    },
    /// A bool literal.
    Bool {
        /// The value of the boolean.
        val: bool,
        /// Trivia data for the boolean.
        trivia: Trivia<'a>,
    },
    /// A datetime literal.
    DateTime {
        /// The value of the date/time.
        val: ChronoDateTime<FixedOffset>,
        /// The original string representation of the value.
        raw: &'a str,
        /// Trivia data for the datetime value.
        trivia: Trivia<'a>,
    },
    /// An array literal.
    Array {
        /// The contents of the array.
        val: Vec<Item<'a>>,
        /// Trivia data for the array.
        trivia: Trivia<'a>,
    },
    /// A table literal.
    Table {
        /// `true` if the table is a member of an AoT.
        is_aot_elem: bool,
        /// The contents of the table.
        val: Container<'a>,
        /// Triva data for the table.
        trivia: Trivia<'a>,
    },
    /// An inline table literal.
    InlineTable {
        /// The contents of the table.
        val: Container<'a>,
        /// Triva data for the table.
        trivia: Trivia<'a>,
    },
    /// A string literal.
    Str {
        /// The type of string this represents
        t: StringType,
        /// The string value
        val: &'a str,
        /// Original string value, including any decoration
        original: &'a str,
        /// Trivia data for the string
        trivia: Trivia<'a>,
    },
    /// An AoT literal.
    AoT(Vec<Vec<Item<'a>>>),
    /// A null item.
    #[doc(hidden)]
    None,
    /// A reference to a segment of an aot earlier in the document
    #[doc(hidden)]
    AoTSegment {
        key: Key<'a>,
        segment: usize,
        payload: Option<Vec<Item<'a>>>,
    },
}

impl<'a> Item<'a> {
    /// Returns the discriminant--a unique integer that represents the type
    /// of the `Item`. This is used for easy comparisons of item types.
    pub fn discriminant(&self) -> i32 {
        use self::Item::*;
        match *self {
            WS(_) => 0,
            Comment(_) => 1,
            Integer { .. } => 2,
            Float { .. } => 3,
            Bool { .. } => 4,
            DateTime { .. } => 5,
            Array { .. } => 6,
            Table { .. } => 7,
            InlineTable { .. } => 8,
            Str { .. } => 9,
            AoT(_) => 10,
            None => 11,
            AoTSegment { .. } => 12,
        }
    }

    pub(crate) fn extend_aot(&mut self, v: Vec<Item<'a>>) -> Result<()> {
        match *self {
            Item::AoT(ref mut val) => {
                val.push(v);
            }
            _ => {
                bail!(ErrorKind::InternalParserError(
                    "Called extend_aot on non-aot variant.".into(),
                ))
            }
        }
        Ok(())
    }

    pub(crate) fn is_homogeneous(&self) -> bool {
        use std::collections::HashSet;
        match *self {
            Item::Array { ref val, .. } => {
                let t = val.iter()
                    .filter_map(|it| match it {
                        &Item::WS(_) |
                        &Item::Comment(_) => None,
                        _ => Some(it.discriminant()),
                    })
                    .collect::<HashSet<_>>()
                    .len();
                t == 1

            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn segment(&self, idx: usize) -> &[Item<'a>] {
        match *self {
            Item::AoT(ref val) => &val[idx],
            _ => {
                println!("{:?}, INDEX: {}", self, idx);
                panic!("Invariant violated: called `segment()` on non-AoT value")
            }
        }
    }

    /// Returns the string representation of an `Item`.
    pub fn as_string(&self) -> String {
        use self::Item::*;
        match *self {
            WS(s) => s.into(),
            Comment(ref meta) => format!("{}{}{}", meta.indent, meta.comment, meta.trail),
            Integer { ref raw, .. } => format!("{}", raw),
            Float { ref raw, .. } => format!("{}", raw),
            Bool { val, .. } => format!("{}", val),
            DateTime { ref raw, .. } => format!("{}", raw),
            Array { ref val, .. } => {
                let mut buf = String::new();
                buf.push_str("[");
                for item in val.iter() {
                    buf.push_str(&item.as_string());
                }
                buf.push_str("]");
                buf
            }
            Table { ref val, .. } => val.as_string(),
            InlineTable { ref val, .. } => {
                let mut buf = String::new();
                buf.push_str("{");
                for (i, &(ref k, ref v)) in val.body.iter().enumerate() {
                    buf.push_str(&format!(
                        "{}{} = {}{}{}",
                        v.trivia().indent,
                        k.clone().unwrap().as_string(),
                        v.as_string(),
                        v.trivia().comment,
                        v.trivia().trail
                    ));
                    if i != val.body.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push_str("}");
                buf
            }
            Str {
                ref t,
                ref original,
                ..
            } => format!("{}{}{}", t.delimiter(), original, t.delimiter()),
            AoT(ref val) => {
                let mut b = String::new();
                for table in &val[0] {
                    b.push_str(&table.as_string());
                }
                b
            }
            None => "".to_string(),
            _ => panic!("Called as_string() on AoTSegment variant."),
        }
    }

    /// Returns a `Trivia`.
    pub fn trivia(&self) -> &Trivia<'a> {
        use self::Item::*;
        match *self {
            WS(_) |
            Comment(_) |
            AoT { .. } |
            AoTSegment { .. } |
            None => {
                println!("{:?}", self);
                panic!("Called trivia on non-value Item variant");
            }
            Integer { ref trivia, .. } |
            Float { ref trivia, .. } |
            Bool { ref trivia, .. } |
            DateTime { ref trivia, .. } |
            Array { ref trivia, .. } |
            Table { ref trivia, .. } |
            InlineTable { ref trivia, .. } |
            Str { ref trivia, .. } => trivia,
        }
    }

    /// Returns a mutable `Trivia`.
    pub fn trivia_mut(&mut self) -> &mut Trivia<'a> {
        use self::Item::*;
        match *self {
            WS(_) |
            Comment(_) |
            AoT { .. } |
            AoTSegment { .. } |
            None => {
                println!("{:?}", self);
                panic!("Called trivia on non-value Item variant");
            }
            Integer { ref mut trivia, .. } |
            Float { ref mut trivia, .. } |
            Bool { ref mut trivia, .. } |
            DateTime { ref mut trivia, .. } |
            Array { ref mut trivia, .. } |
            Table { ref mut trivia, .. } |
            InlineTable { ref mut trivia, .. } |
            Str { ref mut trivia, .. } => trivia,
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn StringType_delimiter() {
        assert_eq!(StringType::SLB.delimiter(), "\"");
        assert_eq!(StringType::MLB.delimiter(), "\"\"\"");
        assert_eq!(StringType::SLL.delimiter(), "'");
        assert_eq!(StringType::MLL.delimiter(), "'''");
    }

    #[test]
    fn Key_delimiter() {
        let mut key = Key::new("myKey");
        assert_eq!(key.t, KeyType::Bare);
        assert_eq!(key.delimiter(), "");
        key.t = KeyType::Basic;
        assert_eq!(key.delimiter(), "\"");
        key.t = KeyType::Literal;
        assert_eq!(key.delimiter(), "'");
    }
}
