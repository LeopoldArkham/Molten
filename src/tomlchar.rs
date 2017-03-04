pub trait TOMLChar {
    // TODO Add validators for each key/value scheme
    // Remove all shameful things
    fn is_bare_key_char(&self) -> bool;
    fn is_ws_or_equal(&self) -> bool;
    fn is_int_float_char(&self) -> bool;
    fn not_whitespace(&self) -> bool;
    fn not_whitespace_or_pound(&self) -> bool;
}

impl TOMLChar for char {
    fn is_bare_key_char(&self) -> bool {
        match *self {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => true,
            _ => false,
        }
    }

    fn is_ws_or_equal(&self) -> bool {
        match *self {
            '=' | ' ' | '\t' => true,
            _ => false,
        }
    }

    fn is_int_float_char(&self) -> bool {
        match *self {
            '+' | '-' | '_' | '0'...'9' | 'e' | '.' => true,
            _ => false,
        }
    }

    fn not_whitespace(&self) -> bool {
        match *self {
            ' ' | '\t' => false,
            _ => true,
        }
    }

    fn not_whitespace_or_pound(&self) -> bool {
        match *self {
            ' ' | '\t' | '#' => false,
            _ => true,
        }
    }
}