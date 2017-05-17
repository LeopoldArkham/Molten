use parser::TLV;

#[derive(Debug, PartialEq)]
pub struct Table {
    pub array: bool,
    pub name: Vec<String>,
    pub comment: String,
    pub values: Vec<TLV>,
}

impl Table {
    pub fn as_string(&self) -> String {
        let mut buf = String::new();
        match self.array {
            true => buf.push_str("[["),
            false => buf.push_str("["),
        }
        let name = self.name.iter().fold(String::new(), |mut acc, ref e| {
            acc.push_str(e);
            return acc;
        });
        // FIXME: Relying on format!() to add newline which the struct
        // should be aware of. Comments haven't been touched.
        // Reuse logic from keyval comment parsing.

        match self.array {
            true => buf.push_str(&format!("{}]]\r\n", name)),
            false => buf.push_str(&format!("{}]\r\n", name)),
        }

        for v in &self.values {
            buf.push_str(&v.as_string());
        }
        buf
    }
}