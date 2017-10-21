use std::ops::Index;
use tomldoc::TOMLDocument;
use items::*;

impl<'a> Index<&'static str> for TOMLDocument<'a> {
    type Output = Item<'a>;

    fn index(&self, name: &str) -> &Self::Output {
        let k = Key {
            t: KeyType::Bare,
            sep: "",
            key: name,
        };
        let idx = self.0.map.get(&k).expect("Invalid key");
        &self.0.body[*idx].1
    }
}

impl<'a> Index<usize> for TOMLDocument<'a> {
    type Output = Item<'a>;

    fn index(&self, idx: usize) -> &Self::Output {
        // XXX TODO
        &self.0.body[idx].1
        // self.0.iter().nth(idx).expect("Indexing TOMLDoc failed")
    }
}

impl<'a> Index<usize> for Item<'a> {
    type Output = Item<'a>;

    fn index(&self, idx: usize) -> &Self::Output {
        use self::Item::*;
        match *self {
            Array { ref val, .. } => &val[idx],
            // XXX TODO
            // Table { ref val, .. } => &val.iter().nth(idx).expect("Indexing Table failed"),
            // InlineTable { ref val, .. } => {
            //     &val.iter().nth(idx).expect("Indexing InlineTable failed")
            // }
            // AoT(ref vec) => &vec.iter().nth(idx).expect("Indexing AoT failed"),
            _ => panic!(),
        }
    }
}

impl<'a> Index<&'static str> for Item<'a> {
    type Output = Item<'a>;

    fn index(&self, name: &str) -> &Self::Output {
        let k = Key {
            t: KeyType::Bare,
            sep: "",
            key: name,
        };

        use self::Item::*;
        match *self {
            Table { ref val, .. } |
            InlineTable { ref val, .. } => &val.body[*val.map.get(&k).expect("Invalid key")].1,
            _ => panic!("Only tables and Inline Tables can be indexed by str"),

        }
    }
}
