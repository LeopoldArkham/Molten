use std::collections::HashMap;

use items::*;

#[derive(Debug)]
pub struct Container {
    // TODO: remove pub after implementing iterator methods
    pub map: HashMap<Key, usize>,
    pub body: Vec<(Option<Key> ,Item)>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            map: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn append<K: Into<Option<Key>>>(&mut self, item: Item, _key: K) {
        let key = _key.into();
        if let Some(k) = key {
            self.map.insert(k, self.body.len());
        }
        self.body.push((key, item));
    }

    // pub fn append(&mut self, key: Key, item: Item) {
    //     if item.is_value() {
    //         self.map.insert(key, self.body.len());
    //     }
    //     self.body.push(item);
    // }

}
