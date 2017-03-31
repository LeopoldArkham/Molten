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

    pub fn as_string(&self) -> String {
        use Value::*;
        match *self {
            SString(ref s) => format!(r#""{}""#, s),
            Integer(ref num) => format!("{}", num),
            Float(ref num) => format!("{}", num),
            Bool(ref b) => format!("{}", b),
            DateTime(ref dt) => format!("{}", dt),
            Array(ref vec) => {
                let mut buf = String::new();
                for val in vec {
                    buf.push_str(&val.as_string());
                    buf.push_str(", ");
                }
                buf
            }
            InlineTable(ref vec) =>{
                let mut buf = String::new();
                for kv in vec {
                    buf.push_str(&kv.as_string());
                    buf.push_str(", ");
                }
                buf
            }
            Table(ref table) => {
                // TODO: Chain names and comment here
                let mut name = table.name.iter()
                                     .fold(String::new(), |mut acc, n| {
                                         acc.push_str(&n);
                                         acc });

                let mut body = String::new();
                for val in &table.values {
                    body.push_str(&val.as_string());
                    body.push_str("\n");
                }
                format!("[{}] {}\n{}", name,table.comment,  body)
            
            }
            WS(ref ws) => ws.clone(),
        }
    }
}