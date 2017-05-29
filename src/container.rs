use std::collections::HashMap;

use items::*;

pub struct Container {
    map: HashMap<i32, i32>,
    body: Vec<Item>,
}