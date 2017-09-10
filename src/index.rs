use std::ops::Index;
use tomldoc::TOMLDocument;
use items::*;

impl Index<&'static str> for TOMLDocument {
    type Output = Item;

    fn index(&self, name: &str) -> &Self::Output {
        let k = Key {
            t: KeyType::Bare,
            raw: name.to_string(),
            actual: name.to_string(),
        };
        let idx = self.0.map.get(&k).expect("Invalid key");
        &self.0.body[*idx].1
    }
}

impl Index<usize> for TOMLDocument {
    type Output = Item;

    fn index(&self, idx: usize) -> &Self::Output {
        self.0.iter().nth(idx).expect("Indexing TOMLDoc failed")
    }
}

impl Index<usize> for Item {
    type Output = Item;

    fn index(&self, idx: usize) -> &Self::Output {
        use self::Item::*;
        match *self {
            Array { ref val, .. } => &val[idx],
            Table { ref val, .. } => &val.iter().nth(idx).expect("Indexing Table failed"),
            InlineTable { ref val, .. } => {
                &val.iter().nth(idx).expect("Indexing InlineTable failed")
            }
            AoT(ref vec) => &vec.iter().nth(idx).expect("Indexing AoT failed"),
            _ => panic!(),
        }
    }
}

impl Index<&'static str> for Item {
    type Output = Item;

    fn index(&self, name: &str) -> &Self::Output {
        let k = Key {
            t: KeyType::Bare,
            raw: name.to_string(),
            actual: name.to_string(),
        };

        use self::Item::*;
        match *self {
            Table { ref val, .. } |
            InlineTable { ref val, .. } => &val.body[*val.map.get(&k).expect("Invalid key")].1,
            _ => panic!("Only tables and Inline Tables can be indexed by str"),

        }
    }
}
