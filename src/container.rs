//! Container to hold items within a `TOMLDocument`.

use std::collections::HashMap;
use std::borrow::Cow;

use items::*;
use errors::*;

/// A container for items within a `TOMLDocument`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Container<'a> {
    pub(crate) map: HashMap<Key<'a>, usize>,
    pub(crate) body: Vec<(Option<Key<'a>>, Item<'a>)>,
}

impl<'a> Container<'a> {
    /// Creates a new empty `Container`.
    pub fn new() -> Container<'a> {
        Container {
            map: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Adds a (key, item) pair to the container.
    pub fn append<K: Into<Option<Key<'a>>>>(&mut self, _key: K, mut item: Item<'a>) -> Result<()> {
        let key = _key.into();
        let mut segment = false;
        if let Some(k) = key.clone() {
            if self.map.contains_key(&k) {
                match item {
                    Item::AoTSegment { ref mut payload, .. } => {
                        println!("Adding AoT segment to container");
                        segment = true;
                        let idx = self.map[&k];
                        self.body[idx].1.extend_aot(payload.take().unwrap())?;
                    }
                    _ => { println!("Not AoTSegment"); bail!(ErrorKind::DuplicateKey(k.key.into()));},
                }
            }
            if !segment {
            self.map.insert(k, self.body.len());
            }
        }
        if segment {
            self.body.push((None, item));            
        } else {
        self.body.push((key, item));
        }
        Ok(())
    }

    /// Removes the kv pair indicated by `key`.
    pub(crate) fn remove(&mut self, key: &Key<'a>) -> Result<()> {
        let idx = self.map.get(key).ok_or(
            ErrorKind::NonExistentKey(key.as_string()),
        )?;
        self.body[*idx] = (None, Item::None);

        Ok(())
    }

    // Returns a mutable reference to the item that was most recently been added to the container.
    // Used for merging whitespace.
    pub(crate) fn last_item_mut(&mut self) -> Option<&mut Item<'a>> {
        self.body.last_mut().map(|&mut (_, ref mut v)| v)
    }

    /// Returns the string representation of a `Container`.
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        for (k, v) in self.body.clone().into_iter() {
            let cur: String = if k.is_some() {
                match v {
                    Item::Table { is_aot_elem, .. } => {
                        let (open, close) = match is_aot_elem {
                            true => ("[[", "]]"),
                            false => ("[", "]"),
                        };
                        format!("{}{}{}{}{}{}{}{}",
                        v.trivia().indent,
                        open,
                        k.unwrap().as_string(),
                        close,
                        v.trivia().comment_ws,
                        v.trivia().comment,
                        v.trivia().trail,
                        v.as_string(),)
                    }
                    Item::AoT (val) => {
                        let mut buf = String::new();
                        let key = k.unwrap().as_string();
                        for table in &val[0] {
                            buf.push_str(&format!(
                                "{}[[{}]]{}{}{}",
                                table.trivia().indent,
                                key,
                                table.trivia().comment_ws,
                                table.trivia().comment,
                                table.trivia().trail
                            ));
                            buf.push_str(&table.as_string());
                        }
                        buf
                    }
                    _ => {
                        let k = k.unwrap();
                        format!(
                            "{}{}{}{}{}{}{}",
                            v.trivia().indent,
                            k.as_string(),
                            k.sep,
                            v.as_string(),
                            v.trivia().comment_ws,
                            v.trivia().comment,
                            v.trivia().trail
                        )
                    }
                }
            } else {
                match v {
                    Item::AoTSegment {key, segment, ..} => {
                        // Needs real key and real ws info
                        let k = Key::new(Cow::Owned(key.clone()));
                        let idx = self.map[&k];
                        let v = self.body[idx].1.segment(segment);
                        let mut buf = String::new();
                        for table in v {
                            buf.push_str(&format!("[[{}]]\r\n", key));
                            buf.push_str(&table.as_string());
                        }
                        buf
                    }
                    _ => v.as_string()
                }
            };
            s.push_str(&cur)
        }
        s
    }

    /// Returns a container iterator.
    pub fn iter(&'a self) -> ContainerIterator<'a> {
        ContainerIterator {
            container: self,
            current: 0,
        }
    }

    /// Returns an exhauseive container iterator.
    pub fn iter_exhaustive(&self) -> ContainerIteratorExhaustive {
        ContainerIteratorExhaustive {
            container: self,
            current: 0,
        }
    }
}

/// An iterator that returns the items in the container.
#[derive(Debug)]
pub struct ContainerIterator<'a> {
    container: &'a Container<'a>,
    current: usize,
}

impl<'a> Iterator for ContainerIterator<'a> {
    type Item = &'a Item<'a>;

    // TODO: There must be a better way
    fn next(&mut self) -> Option<&'a Item<'a>> {
        loop {
            if self.current == self.container.body.len() {
                return None;
            }
            match self.container.body[self.current].1.is_value() {
                true => {
                    let r = Some(&self.container.body[self.current].1);
                    self.current += 1;
                    return r;
                }
                false => self.current += 1,
            }
        }
    }
}

/// An iterator that returns **all** items in the container.
#[derive(Debug)]
pub struct ContainerIteratorExhaustive<'a> {
    container: &'a Container<'a>,
    current: usize,
}

impl<'a> Iterator for ContainerIteratorExhaustive<'a> {
    type Item = &'a Item<'a>;

    fn next(&mut self) -> Option<&'a Item<'a>> {
        if self.current != self.container.body.len() {
            let r = &self.container.body[self.current];
            self.current += 1;
            Some(&r.1)
        } else {
            None
        }
    }
}
