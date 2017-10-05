#![allow(non_snake_case)]
#[allow(unused_imports)]

#[macro_use]
extern crate pretty_assertions;
extern crate chrono;

mod tomlchar;
mod tomldoc;
pub mod parser;
mod index;
mod items;
mod container;