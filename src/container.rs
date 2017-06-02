use std::collections::HashMap;

use items::*;

#[derive(Debug)]
pub struct Container {
    map: HashMap<Key, usize>,
    body: Vec<Item>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            map: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn append(&mut self, item: Item, key: Option<Key>) {
        if let Some(k) = key {
            self.map.insert(k, self.body.len());
        }
        self.body.push(item);
    }

    // pub fn append(&mut self, key: Key, item: Item) {
    //     if item.is_value() {
    //         self.map.insert(key, self.body.len());
    //     }
    //     self.body.push(item);
    // }

}
