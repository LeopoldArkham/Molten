use value::Value;
use parser::TLV;

#[derive(Debug)]
pub struct TOMLDocument(pub Vec<TLV>);

impl TOMLDocument {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        for elem in self.0 {
            buf.push_str(&elem.as_string());
        }
        buf
    }
}