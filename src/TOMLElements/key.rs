#[derive(Debug, PartialEq)]
pub enum Key {
    Bare(String),
    Quoted(String),
}

impl Key {
    fn bare<T: Into<String>>(name: T) -> Key {
        Key::Bare(name.into())
    }

    fn quoted<T: Into<String>>(name: T) -> Key {
        Key::Quoted(name.into())
    }

    pub fn as_string(&self) -> String {
        match *self {
            Key::Bare(ref s) => s.clone(),
            Key::Quoted(ref s) => s.clone(),
        }
    }
}