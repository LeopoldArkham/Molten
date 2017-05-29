use std::collections::HashMap;

use items::*;

pub struct Container {
    map: HashMap<i32, i32>,
    body: Vec<Item>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            map: HashMap::new(),
            body: Vec::new,
        }
    }

    pub fn append(&mut self, key: Key, item: Item) {
        if item.is_value() {
            self.map.insert(key, self.vec.len());
        }
        self.vec.push(item);
    }

}

