use chrono::{DateTime as ChronoDateTime, FixedOffset};

use Table;
use KeyValue;

#[derive(Debug, PartialEq)]
pub enum Value {
    SString(String), // Quote
    Integer(i64), // Digit, +, -
    Float(f64), // Digit, +, -
    Bool(bool), // char
    DateTime(ChronoDateTime<FixedOffset>), // Digit
    Array(Vec<Value>), // Bracket
    InlineTable(Vec<KeyValue>), // Curly bracket
    Table(Table),
    WS(String),
}

impl Value {
    // TODO: usize required here?
    pub fn discriminant(&self) -> usize {
        use Value::*;
        match *self {
            // TODO: use self::...
            SString(_) => 1 as usize,
            Integer(_) => 2 as usize,
            Float(_) => 3 as usize,
            Bool(_) => 4 as usize,
            DateTime(_) => 5 as usize,
            Array(_) => 6 as usize,
            InlineTable(_) => 7 as usize,
            Table(_) => 8 as usize,
            WS(_) => 9 as usize,
        }
    }
}