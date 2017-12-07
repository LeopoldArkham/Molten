//! TOML document representation

use container::Container;
use items::*;
use errors::*;

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
    
    /// Appends a (key, value) pair to the document.
    pub fn append<K>(&mut self, _key: K, item: Item<'a>) -> Result<()>
    where K: Into<Option<Key<'a>>> {
        // TODO: Tables and aot's sould be appended normally;
        // all other items should be inserted before the first table-like
        // item.
        self.0.append(_key, item)
    }
}
