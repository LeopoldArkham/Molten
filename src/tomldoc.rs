use container::Container;

#[derive(Debug)]
pub struct TOMLDocument(pub Vec<Container>);

impl TOMLDocument {
    pub fn as_string(&self) -> String {
        let mut buf = String::new();
        for elem in &self.0 {
            buf.push_str(&elem.as_string());
        }
        buf
    }
}
