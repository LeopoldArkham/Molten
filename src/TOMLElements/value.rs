use chrono::{DateTime as ChronoDateTime, FixedOffset};

use super::Table;
use super::KeyValue;
use super::StrEnum;

#[derive(Debug, PartialEq)]
pub enum Value {
    Str(StrEnum), // Quote
    Integer(i64), // Digit, +, -
    Float(f64), // Digit, +, -
    Bool(bool), // char
    DateTime((ChronoDateTime<FixedOffset>, String)), // Digit
    Array(Vec<Value>), // Bracket
    InlineTable(Vec<KeyValue>), // Curly bracket
    Table(Table),
}

impl Value {
    pub fn discriminant(&self) -> u32 {
        use self::Value::*;
        match *self {
            // TODO: use self::...
            Str(_) => 1 as u32,
            Integer(_) => 2 as u32,
            Float(_) => 3 as u32,
            Bool(_) => 4 as u32,
            DateTime(_) => 5 as u32,
            Array(_) => 6 as u32,
            InlineTable(_) => 7 as u32,
            Table(_) => 8 as u32,
        }
    }

    pub fn as_string(&self) -> String {
        use self::Value::*;
        match *self {
            Str(ref s) => s.as_string(),
            Integer(ref num) => format!("{}", num),
            Float(ref num) => format!("{}", num),
            Bool(ref b) => format!("{}", b),
            DateTime((dt, ref s)) => s.clone(),
            Array(ref vec) => {
                let mut buf = String::new();
                buf.push_str("[");
                for (i, val) in vec.iter().enumerate() {
                    buf.push_str(&val.as_string());
                    if i != vec.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push_str("]");
                buf
            }
            InlineTable(ref vec) => {
                let mut buf = String::new();
                for kv in vec {
                    buf.push_str(&kv.as_string());
                    buf.push_str(", ");
                }
                buf
            }
            Table(ref table) => {
                // TODO: Chain names and comment here
                let name = table.name
                    .iter()
                    .fold(String::new(), |mut acc, n| {
                        acc.push_str(&n);
                        acc
                    });

                let mut body = String::new();
                for val in &table.values {
                    body.push_str(&val.as_string());
                    body.push_str("\n");
                }
                format!("[{}] {}\n{}", name, table.comment, body)
            }
        }
    }
}
