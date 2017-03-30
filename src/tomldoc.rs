use value::Value;
use parser::TLV;

#[derive(Debug)]
pub struct TOMLDocument(pub Vec<TLV>);