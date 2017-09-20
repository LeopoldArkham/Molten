#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]

#[macro_use]
extern crate pretty_assertions;
extern crate chrono;

mod tomlchar;
mod tomldoc;
pub mod parser;
mod index;
mod items;
mod container;
mod comment;