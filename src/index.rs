use std::ops::Index;
use tomldoc::TOMLDocument;
use items::*;

// impl Index<&'static str> for TOMLDocument {
//     type Output =  TLV;

//     fn index(&self, name: &str) -> &Self::Output {
//         unimplemented!();
//     }
// }

impl Index<usize> for TOMLDocument {
    type Output =  Item;

    fn index(&self,  idx: usize) -> &Self::Output {
        self.0.iter().nth(idx).expect("Indexing TOMLDoc failed")
    }
}

impl Index<usize> for Item {
    type Output =  Item;

    fn index(&self,  idx: usize) -> &Self::Output {
        use self::Item::*;
        match *self {
            Array{ref val, ..} => {
                &val[idx]
            }
            Table{ref val, ..} => {
                &val.iter().nth(idx).expect("Indexing Table failed")
            }
            InlineTable{ref val, ..} => {
                &val.iter().nth(idx).expect("Indexing InlineTable failed")
            }
            AoT(ref vec) => {
                &vec.iter().nth(idx).expect("Indexing AoT failed")
            }
            _ => panic!()
        }
    }
}

// pub trait IndexReturn: Debug {}
// impl IndexReturn for Value {}
// impl IndexReturn for KeyValue {}

// impl Index<usize> for Value {
//     type Output = IndexReturn;

//     fn index(&self, idx: usize) -> &Self::Output {
//         use self::Value::*;

//         match *self {
//             Array(ref v) => return &v[idx],
//             InlineTable(ref v) => return &v[idx],
//             _ => panic!("Can only index Arrays and Inline Tables.")
//         }
//     }
// }

// impl Index<usize> for KeyValue {
//     type Output = IndexReturn;

//     fn index(&self, idx: usize) -> &Self::Output {
//         use self::Value::*;

//         match self.value {
//             Array(ref v) => return &v[idx],
//             InlineTable(ref v) => return &v[idx],
//             _ => panic!("Can only index Arrays and Inline Tables.")
//         }
//     }
// }

// impl Index<usize> for TLV {
//     type Output = IndexReturn;

//     fn index(&self, idx: usize) -> &Self::Output {
//         use self::TLV::*;
        
//         match *self {
//             Val(ref kv) => return &kv[idx],
//             _ => unimplemented!(),
//         }
//     }
// }