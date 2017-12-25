//! TOML document representation

use container::Container;


/// A TOML document.
///
/// This is the container that holds the contents of a TOML file.
pub type TOMLDocument<'a> = Container<'a>;
