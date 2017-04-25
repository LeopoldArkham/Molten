use super::Value;
use super::Comment;
use super::Key;

#[derive(Debug, PartialEq)]
pub struct KeyValue {
    pub indent: String,
    pub key: Key,
    pub value: Value,
    pub comment: Option<Comment>,
    pub trail: String,
}

impl KeyValue {
    pub fn as_string(&self) -> String {
        let mut buf = String::new();
        buf.push_str(&self.indent);
        buf.push_str(&self.key.as_string());
        buf.push_str(" = ");
        buf.push_str(&self.value.as_string());
        if let Some(ref _comment) = self.comment {
            buf.push_str(&_comment.as_string());
        }
        buf.push_str(&self.trail);
        buf
    }
}