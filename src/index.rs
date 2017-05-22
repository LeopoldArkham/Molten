use std::ops::Index;
use std::fmt;

use tomldoc::TOMLDocument;
use TOMLElements::*;
use parser::TLV;

impl Index<&'static str> for TOMLDocument {
    type Output =  TLV;

    fn index(&self, name: &str) -> &Self::Output {
        unimplemented!();
    }
}

impl Index<usize> for TOMLDocument {
    type Output =  TLV;

    fn index(&self,  idx: usize) -> &Self::Output {
        use self::TLV::*;
        let mut cur = 0;
        let mut skipped = 0;

        for elem in &self.0 {
            match elem {
                &WS(_) | &Comment(_) => {
                    cur += 1;
                    skipped += 1;
                }
                _ => {
                    if cur - skipped == idx {
                       break;
                    } else {
                        cur += 1
                    }
                }
            }
        }
        &self.0[cur]
    }
}

pub trait IndexReturn {}
impl IndexReturn for Value {}
impl IndexReturn for KeyValue {}

impl Index<usize> for Value {
    type Output = IndexReturn;

    fn index(&self, idx: usize) -> &Self::Output {
        use self::Value::*;

        match *self {
            Array(ref v) => return &v[idx],
            InlineTable(ref v) => return &v[idx],
            _ => panic!("Can only index Arrays and Inline Tables.")
        }
    }
}

impl Index<usize> for KeyValue {
    type Output = IndexReturn;

    fn index(&self, idx: usize) -> &Self::Output {
        use self::Value::*;

        match self.value {
            Array(ref v) => return &v[idx],
            InlineTable(ref v) => return &v[idx],
            _ => panic!("Can only index Arrays and Inline Tables.")
        }
    }
}

impl Index<usize> for TLV {
    type Output = IndexReturn;

    fn index(&self, idx: usize) -> &Self::Output {
        use self::TLV::*;
        
        match *self {
            Val(ref kv) => return &kv[idx],
            _ => unimplemented!(),
        }
    }
}