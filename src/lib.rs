#![allow(non_snake_case)]
#![feature(inclusive_range_syntax, range_contains)]

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
mod comment;