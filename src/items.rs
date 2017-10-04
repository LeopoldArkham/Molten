
use chrono::{DateTime as ChronoDateTime, FixedOffset};
use container::Container;
use comment::Comment;

#[derive(Debug, Clone)]
pub enum StringType {
    SLB,
    /// The multi-line basic string's in-file representation
    /// can differ from what it actually represents, so we store
    /// the raw string here
    MLB(String),
    SLL,
    MLL,
}

#[derive(Debug, Default, Clone)]
pub struct LineMeta {
    pub indent: String,
    pub comment: Option<Comment>,
    pub trail: String,
}

impl LineMeta {
    pub fn comment(&self) -> String {
        if let Some(ref c) = self.comment {
            c.as_string()
        } else {
            "".to_string()
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum KeyType {
    Bare,
    Basic,
    Literal,
}

#[derive(Hash, Clone)]
pub struct Key {
    pub t: KeyType,
    pub sep: String,
    pub key: String,
}

impl Eq for Key {}

impl PartialEq for Key {
    fn eq(&self, other: &Key) -> bool {
        self.key == other.key
    }
}

impl ::std::fmt::Debug for Key {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl Key {
    pub fn as_string(&self) -> String {
        let quote = match self.t {
            KeyType::Bare => "",
            KeyType::Basic => "\"",
            KeyType::Literal => "'",
        };

        format!("{}{}{}", quote, self.key, quote)
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    // @todo: Move comment struct content here. Also display logic
    WS(String),
    Comment(Comment),
    Integer {
        val: i64,
        meta: LineMeta,
        raw: String,
    },
    Float {
        val: f64,
        meta: LineMeta,
        raw: String,
    },
    Bool { val: bool, meta: LineMeta },
    DateTime {
        val: ChronoDateTime<FixedOffset>,
        raw: String,
        meta: LineMeta,
    },
    Array { val: Vec<Item>, meta: LineMeta },
    Table {
        is_array: bool,
        val: Container,
        meta: LineMeta,
    },
    InlineTable { val: Container, meta: LineMeta },
    Str {
        t: StringType,
        val: String,
        meta: LineMeta,
    },
    AoT(Vec<Item>),
}

impl Item {
    pub fn is_value(&self) -> bool {
        use self::Item::*;
        match *self {
            WS(_) | Comment(_) => false,
            _ => true,
        }
    }

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
        }
    }

    pub fn is_table(&self) -> bool {
        match self.discriminant() {
            7 | 10 => true,
            _ => false,
        }
    }

    pub(crate) fn is_homogeneous(&self) -> bool {
        use std::collections::HashSet;
        match *self {
            Item::Array { ref val, .. } => {
                let t = val.iter()
                    .filter_map(|it| {
                        match it {
                            &Item::WS(_) |
                            &Item::Comment(_) => None,
                            _ => Some(it.discriminant()),
                        }
                    })
                    .collect::<HashSet<_>>()
                    .len();
                t == 1

            }
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> String {
        use self::Item::*;
        match *self {
            WS(ref s) => s.clone(),
            Comment(ref c) => c.as_string(),
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
                    buf.push_str(&format!("{}{} = {}{}{}",
                                          v.meta().indent,
                                          k.clone().unwrap().as_string(),
                                          v.as_string(),
                                          v.meta().comment(),
                                          v.meta().trail));
                    if i != val.body.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push_str("}");
                buf
            }
            Str { ref t, ref val, .. } => {
                match *t {
                    StringType::MLB(ref s) => format!(r#"{}"#, s),
                    StringType::SLB => format!(r#""{}""#, val),
                    StringType::SLL => format!(r#"'{}'"#, val),
                    StringType::MLL => format!(r#"'''{}'''"#, val),
                }
            }
            AoT(ref body) => {
                let mut b = String::new();
                for table in body {
                    b.push_str(&table.as_string());
                }
                b
            }
        }
    }

    pub fn meta<'a>(&'a self) -> &'a LineMeta {
        use self::Item::*;
        match *self {
            WS(_) | Comment(_) | AoT(_) => {
                println!("{:?}", self);
                panic!("Called meta on non-value Item variant");
            }
            Integer { ref meta, .. } |
            Float { ref meta, .. } |
            Bool { ref meta, .. } |
            DateTime { ref meta, .. } |
            Array { ref meta, .. } |
            Table { ref meta, .. } |
            InlineTable { ref meta, .. } |
            Str { ref meta, .. } => meta,
        }
    }

    pub fn meta_mut<'a>(&'a mut self) -> &'a mut LineMeta {
        use self::Item::*;
        match *self {
            WS(_) | Comment(_) | AoT(_) => {
                println!("{:?}", self);
                panic!("Called meta on non-value Item variant");
            }
            Integer { ref mut meta, .. } |
            Float { ref mut meta, .. } |
            Bool { ref mut meta, .. } |
            DateTime { ref mut meta, .. } |
            Array { ref mut meta, .. } |
            Table { ref mut meta, .. } |
            InlineTable { ref mut meta, .. } |
            Str { ref mut meta, .. } => meta,
        }
    }
}
