//! Index methods for varous types

use std::ops::{Index, IndexMut};
use container::Container;
use items::*;


impl<'a> Index<usize> for Container<'a> {
    type Output = Item<'a>;

    fn index(&self, idx: usize) -> &Self::Output {
        let mut current = 0;
        for item in &self.body {
            if item.0.is_some() {
                if current == idx {
                    return &item.1;
                } else {
                    current += 1;
                }
            }
        }
        panic!("Index out of bounds");
    }
}

impl<'a> IndexMut<usize> for Container<'a> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        let mut current = 0;
        for item in &mut self.body {
            if item.0.is_some() {
                if current == idx {
                    return &mut item.1;
                } else {
                    current += 1;
                }
            }
        }
        panic!("Index out of bounds");
    }
}

impl<'a> Index<&'static str> for Container<'a> {
    type Output = Item<'a>;

    fn index(&self, name: &str) -> &Self::Output {
        let k = Key {
            t: KeyType::Bare,
            sep: "",
            key: name.into(),
            raw: name.into(),
        };
        let idx = self.map.get(&k).expect("Invalid key");
        &self.body[*idx].1
    }
}

impl<'a> IndexMut<&'static str> for Container<'a> {
    fn index_mut(&mut self, name: &str) -> &mut Self::Output {
        let k = Key {
            t: KeyType::Bare,
            sep: "",
            key: name.into(),
            raw: name.into(),            
        };
        let idx = self.map.get(&k).expect("Invalid key");
        &mut self.body[*idx].1
    }
}

impl<'a> Index<usize> for Item<'a> {
    type Output = Item<'a>;

    fn index(&self, idx: usize) -> &Self::Output {
        use self::Item::*;
        match *self {
            Array { ref val, .. } => {
                let mut current = 0;
                for item in val {
                    if item.is_value() {
                        if current == idx {
                            return item;
                        } else {
                            current += 1;
                        }
                    }
                }
                panic!("Index out of bounds");
            }
            Table { ref val, .. } => &val[idx],
            InlineTable { ref val, .. } => &val[idx],
            // @fixme: indexing AoTs
            // AoT (val) => &val.iter().nth(idx).expect("Indexing AoT failed"),
            AoT (_) => {
                
            },
            _ => panic!("This value cannot be indexed."),
        }
    }
}

impl<'a> Index<&'static str> for Item<'a> {
    type Output = Item<'a>;

    fn index(&self, name: &str) -> &Self::Output {
        let k = Key {
            t: KeyType::Bare,
            sep: "",
            key: name.into(),
            raw: name.into(),                        
        };

        use self::Item::*;
        match *self {
            Table { ref val, .. } |
            InlineTable { ref val, .. } => &val.body[*val.map.get(&k).expect("Invalid key")].1,
            _ => panic!("Only tables and Inline Tables can be indexed by str"),
        }
    }
}
