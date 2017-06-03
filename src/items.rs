
use chrono::{DateTime as ChronoDateTime, FixedOffset};
use container::Container;
use comment::Comment;

#[derive(Debug)]
pub enum StringType {
    SLB,
    /// The multi-line basic string's in-file representation
    /// can differ from what it actually represents, se we store
    /// the raw string here
    MLB(String),
    SLL,
    MLL,
}

#[derive(Debug, Default)]
pub struct LineMeta {
    indent: String,
    comment: Option<Comment>,
    trail: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
// TODO: Store raw in enum variant
pub enum KeyType {
    Bare,
    Quoted,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Key {
    t: KeyType,
    raw: String,
    actual: String,
}

#[derive(Debug)]
// TODO: Add LineMeta field to all value variants
pub enum Item {
    WS(String),
    Comment(Comment),
    Integer {
      val: i64,
      meta: LineMeta,  
    },
    Float {
     val: f64,
     meta: LineMeta,   
    },
    Bool {
        val: bool,
        meta: LineMeta,
    },
    DateTime {
        val: ChronoDateTime<FixedOffset>,
        raw: String,
        meta: LineMeta,
    },
    Array {
        val: Vec<Item>,
        meta: LineMeta,
    },
    Table {
        is_array: bool,
        val: Container,
        meta: LineMeta,
    },
    InlineTable {
        val: Container,
        meta: LineMeta,
    },
    Str {
        t: StringType,
        val: String,
        meta: LineMeta,
    },
    AoT(Vec<Item>)
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
        // TODO: Move comment struct content here. Also display logic
        match *self {
            WS(_) => 0,
            Comment(_) => 1,
            Integer {..} => 2,
            Float {..} => 3,
            Bool {..} => 4,
            DateTime {..} => 5,
            Array {..} => 6,
            Table {..} => 7,
            InlineTable {..} => 8,
            Str {..} => 9,
            AoT(_) => 10,
        }
    }

    pub fn as_string(&self) -> String {
        use self::Item::*;
        match *self {
            WS(s) => s,
            Comment(c) => c.as_string(),
            Integer{val, meta} => {
                let c = comment_to_string(meta.comment);
                format!{"{}{}{}{}", meta.indent, val, c, meta.trail}
            },
            Float{val, meta} => {
                let c = comment_to_string(meta.comment);
                format!{"{}{}{}{}", meta.indent, val, c, meta.trail}
            },
            Bool{val, meta} => {
                let c = comment_to_string(meta.comment);
                format!{"{}{}{}{}", meta.indent, val, c, meta.trail}
            },
            DateTime{raw, meta, ..} => {
                let c = comment_to_string(meta.comment);
                format!{"{}{}{}{}", meta.indent, raw, c, meta.trail}
            },
            Array{val, meta} => {
                let c = comment_to_string(meta.comment);
                let mut buf = String::new();
                buf.push_str("[");
                for item in val {
                    buf.push_str(&item.as_string());
                    buf.push_str(", ");
                }
                buf.push_str("]");
                format!{"{}{}{}{}", meta.indent, buf, c, meta.trail}
            },
            Table{val, is_array, meta} => {
                "".to_string()
            }
            InlineTable{val, meta} => {
                let c = comment_to_string(meta.comment);
                let mut buf = String::new();
                buf.push_str("{");
                for (k, v) in val.body {
                    buf.push_str(&item.as_string());
                    buf.push_str(", ");
                }
                buf.push_str("]");
                format!{"{}{}{}{}", meta.indent, buf, c, meta.trail}
            }
        } 
    }
}

fn comment_to_string(c: Option<Comment>)-> String {
    if let Some(com) = c {
        com.as_string()
    } else {
        "".to_string()
    }
}