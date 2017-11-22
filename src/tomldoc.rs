use container::Container;

/// A TOML document.
///
/// This is the container that holds the contents of a TOML file.
#[derive(Debug, PartialEq)]
pub struct TOMLDocument<'a>(pub Container<'a>);

impl<'a> TOMLDocument<'a> {
    /// Return the string reprentation of a `TOMLDocument`.
    pub fn as_string(&self) -> String {
        self.0.as_string()
    }
}
