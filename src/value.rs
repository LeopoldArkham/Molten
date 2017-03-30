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
        match *self {
            // TODO: use self::...
            Value::SString(_) => 1 as usize,
            Value::Integer(_) => 2 as usize,
            Value::Float(_) => 3 as usize,
            Value::Bool(_) => 4 as usize,
            Value::DateTime(_) => 5 as usize,
            Value::Array(_) => 6 as usize,
            Value::InlineTable(_) => 7 as usize,
            Value::Table(_) => 8 as usize,
            Value::WS(_) => 9 as usize,
        }
    }
}