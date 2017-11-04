#![allow(non_snake_case)]
#[allow(unused_imports)]

#[macro_use]
extern crate pretty_assertions;
extern crate chrono;
#[macro_use]
extern crate error_chain;

mod tomlchar;
pub mod tomldoc;
pub mod parser;
mod api;
mod index;
mod items;
pub mod container;
mod errors;

pub use tomldoc::TOMLDocument;
pub use container::Container;