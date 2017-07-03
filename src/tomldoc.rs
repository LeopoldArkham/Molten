use container::Container;

#[derive(Debug)]
pub struct TOMLDocument(pub Container);

impl TOMLDocument {
    pub fn as_string(&self) -> String {
        self.0.as_string()
    }
}
