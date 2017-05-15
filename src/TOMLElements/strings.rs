#[derive(Debug, PartialEq)]
pub enum StrEnum {
    /// Single-Line Basic String
    SLBString(String),
    /// Multi-Line Basic String
    MLBString(MLString),
    /// Single-Line Literal String
    SLLString(String),
    /// Multi-Line Literal String
    MLLString(String),
}

#[derive(Debug, PartialEq)]
pub struct MLString {
    pub actual: String,
    pub raw: String,
}

impl StrEnum {
    pub fn as_string(&self) -> String {
        match *self {
            StrEnum::SLBString(ref s) => {
                format!(r#""{}""#, s.clone())
            }
            StrEnum::SLLString(ref s) => {
                format!("'{}'", s.clone())
            }
            StrEnum::MLLString(ref s) => {
                format!("'''{}'''", s.clone())
            }
            StrEnum::MLBString(ref ml) => {
                ml.raw.clone()
            }
        }
    }
}