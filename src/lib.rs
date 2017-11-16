#![allow(non_snake_case)]
#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;
extern crate chrono;
#[macro_use]
extern crate error_chain;

mod tomlchar;
mod errors;
pub mod tomldoc;
pub mod parser;
pub mod api;
pub mod index;
pub mod items;

pub use tomldoc::TOMLDocument;
pub use items::*;

// Only public as a hack for testing;
// Should be private and handled via API
pub mod container;
pub use container::Container;

#[cfg(windows)] pub const NL: &'static str = "\r\n";
#[cfg(not(windows))] pub const NL: &'static str = "\n";
