#[derive(Debug, PartialEq)]
pub struct Comment {
    indent: String,
    comment: String,
}

impl Comment {
    pub fn as_string(&self) -> String {
        format!("{}#{}", self.indent, self.comment)
    }
}