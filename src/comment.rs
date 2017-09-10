#[derive(Debug, PartialEq, Clone)]
pub struct Comment {
    pub indent: String,
    pub comment: String,
}

impl Comment {
    pub fn as_string(&self) -> String {
        format!("{}#{}", self.indent, self.comment)
    }
}
