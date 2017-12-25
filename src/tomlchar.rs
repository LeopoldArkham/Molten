//! Classification methods used by `Parser` when parsing text.

/// Classification methods used when parsing TOML text.
///
/// **Note:** these functions return `true` if the given character is in the
/// valid set of chracters allowed for a specific class. Some characters (e.x.:,
/// '_', 'e', and '.' in a float) may be valid, but only within certain contexts
/// within the string representation of that class. The caller is responsible
/// for determining whether the character is valid in contexxt, since these
/// functions have no access to the surrounding context.
///
/// This trait is sealed and cannot be implmeneted for types outside this crate.
pub trait TOMLChar {
    #[doc(hidden)]
    /// Is the character a valid bare key name character?
    fn is_bare_key_char(&self) -> bool;

    #[doc(hidden)]
    /// Is the character a valid key value separator?
    fn is_kv_sep(&self) -> bool;

    #[doc(hidden)]
    /// Is the character a valid integer or float value character?
    fn is_int_float_char(&self) -> bool;

    #[doc(hidden)]
    /// Is the character whitespace?
    fn is_ws(&self) -> bool;

    #[doc(hidden)]
    /// Is the character a line separator?
    fn is_nl(&self) -> bool;

    #[doc(hidden)]
    /// Is the character a space?
    fn is_spaces(&self) -> bool;

    #[doc(hidden)]
    /// Return `true` if the character is not contained in the string `str`.
    fn not_in(&self, list: &str) -> bool;
}

mod private {
    /// `Sealed` stops crates other than Molten from implementing any traits
    /// that use it. This allows us to change the trait API without introducing
    /// a breaking change.
    ///
    /// For more information see the [Rust API Guidelines]
    /// (https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html).
    pub trait Sealed {}
    impl Sealed for super::TOMLChar {}
}

impl TOMLChar for char {
    fn is_bare_key_char(&self) -> bool {
        match *self {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => true,
            _ => false,
        }
    }

    fn is_kv_sep(&self) -> bool {
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

    fn is_ws(&self) -> bool {
        match *self {
            ' ' | '\t' | '\r' | '\n' => true,
            _ => false,
        }
    }

    fn is_nl(&self) -> bool {
        match *self {
            '\n' | '\r' => true,
            _ => false,
        }
    }

    fn is_spaces(&self) -> bool {
        match *self {
            ' ' | '\t' => true,
            _ => false,
        }
    }

    fn not_in(&self, list: &str) -> bool {
        for ch in list.chars() {
            if *self == ch {
                return false;
            }
        }
        true
    }
}
