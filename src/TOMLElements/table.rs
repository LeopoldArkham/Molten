#[derive(Debug, PartialEq)]
pub struct Table {
    name: Vec<String>,
    comment: String,
    values: Vec<parser::TLV>,
}

impl Table {
    pub fn as_string(&self) -> String {
        let mut buf = String::new();
        buf.push_str("[");
        let name = self.name.iter().fold(String::new(), |mut acc, ref e| {
            acc.push_str(e);
            return acc;
        });
        // FIXME: Relying on format!() to add newline which the struct
        // should be aware of. Comments haven't been touched.
        // Reuse logic from keyval comment parsing.
        buf.push_str(&format!("{}]\r\n", name));

        for v in &self.values {
            buf.push_str(&v.as_string());
        }
        buf
    }
}