
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
        value: Container,
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
}