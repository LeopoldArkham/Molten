use container::Container;

#[derive(Debug)]
pub struct TOMLDocument<'a>(pub Container<'a>);

impl<'a> TOMLDocument<'a> {
    pub fn as_string(&self) -> String {
        self.0.as_string()
    }
}
