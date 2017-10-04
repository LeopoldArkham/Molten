use std::collections::HashMap;

use items::*;

#[derive(Debug, Clone)]
pub struct Container {
    pub(crate) map: HashMap<Key, usize>,
    pub(crate) body: Vec<(Option<Key>, Item)>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            map: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn append<K: Into<Option<Key>>>(&mut self, _key: K, item: Item) -> Result<(), String> {
        let key = _key.into();
        if let Some(k) = key.clone() {
            if self.map.contains_key(&k) {
                return Err(format!("Cannot override existing key: {:?}", k));
            }
            self.map.insert(k, self.body.len());
        }
        self.body.push((key, item));
        Ok(())
    }

    // @cleanup: minimize duplication with Item::as_string()
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        for (k, v) in self.body.clone().into_iter() {
            let cur: String = if k.is_some() {
                match v {
                    Item::Table { is_array, .. } => {
                        let (open, close) = match is_array {
                            true => ("[[", "]]"),
                            false => ("[", "]"),
                        };
                        format!("{}{}{}{}{}{}{}",
                        v.meta().indent,
                        open,
                        k.unwrap().as_string(),
                        close,
                        v.meta().comment(),
                        v.meta().trail,
                        v.as_string(),)
                    }
                    Item::AoT(vec) => {
                        let mut buf = String::new();
                        let key = k.unwrap().as_string();
                        for table in vec {
                            buf.push_str(&format!("{}[[{}]]{}{}",
                                                  table.meta().indent,
                                                  key,
                                                  table.meta().comment(),
                                                  table.meta().trail));
                            buf.push_str(&table.as_string());
                        }
                        buf
                    }
                    _ => {
                        let k = k.unwrap();
                        format!("{}{}{}{}{}{}",
                                v.meta().indent,
                                k.as_string(),
                                k.sep,
                                v.as_string(),
                                v.meta().comment(),
                                v.meta().trail)
                    }
                }
            } else {
                v.as_string()
            };
            s.push_str(&cur)
        }
        s
    }

    pub fn iter(&self) -> ContainerIterator {
        ContainerIterator {
            container: self,
            current: 0,
        }
    }

    pub fn iter_exhaustive(&self) -> ContainerIteratorExhaustive {
        ContainerIteratorExhaustive {
            container: self,
            current: 0,
        }
    }
}

pub struct ContainerIterator<'a> {
    container: &'a Container,
    current: usize,
}

impl<'a> Iterator for ContainerIterator<'a> {
    type Item = &'a Item;

    // @cleanup: "There must be a better way"
    fn next(&mut self) -> Option<&'a Item> {
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

pub struct ContainerIteratorExhaustive<'a> {
    container: &'a Container,
    current: usize,
}

impl<'a> Iterator for ContainerIteratorExhaustive<'a> {
    type Item = &'a Item;

    fn next(&mut self) -> Option<&'a Item> {
        if self.current != self.container.body.len() {
            let r = &self.container.body[self.current];
            self.current += 1;
            Some(&r.1)
        } else {
            None
        }
    }
}
