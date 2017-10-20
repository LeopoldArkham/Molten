#![allow(non_snake_case)]
#[allow(unused_imports)]

#[macro_use]
extern crate pretty_assertions;
extern crate chrono;
#[macro_use]
extern crate error_chain;

mod tomlchar;
mod tomldoc;
pub mod parser;
mod api;
mod index;
mod items;
mod container;
mod errors;
