use items::*;
use container::*;
use errors::*;

pub fn table<'a>() -> Item<'a> {
    Item::Table {
        is_array: false,
        val: Container::new(),
        meta: Trivia::empty(),
    }
}

impl<'a> Item<'a> {
    pub fn append<K: Into<Option<Key<'a>>>>(&mut self, _key: K, item: Item<'a>) -> Result<()> {
        use Item::*;
        match *self {
            Table {ref mut val, ..} | InlineTable {ref mut val, ..} => val.append(_key, item),
            Array {..} | AoT {..} => unimplemented!(),
            _ => bail!(ErrorKind::APIWrongItem),
        }
    }
}

impl<'a> Item<'a> {
    /// Returns true if Item is a value.
    pub fn is_value(&self) -> bool {
        match self.discriminant() {
            0 | 1 => false,
            _ => true,
        }
    }

    /// Returns true if Item is trivia.
    pub fn is_trivia(&self) -> bool {
        !self.is_value()
    }

    /// Returns true if Item is whitespace.
    pub fn is_ws(&self) -> bool {
        self.discriminant() == 0
    }

    /// Returns true if Item is a comment.
    pub fn is_comment(&self) -> bool {
        self.discriminant() == 1
    }

    /// Returns true if Item is an integer.
    pub fn is_integer(&self) -> bool {
        self.discriminant() == 2
    }

    /// Returns true if Item is a float.
    pub fn is_float(&self) -> bool {
        self.discriminant() == 3
    }

    /// Returns true if Item is a boolean.
    pub fn is_bool(&self) -> bool {
        self.discriminant() == 4
    }

    /// Returns true if Item is a date/time.
    pub fn is_date_time(&self) -> bool {
        self.discriminant() == 5
    }

    /// Returns true if Item is an array.
    pub fn is_array(&self) -> bool {
        self.discriminant() == 6
    }

    /// Returns true if Item is a table.
    pub fn is_table(&self) -> bool {
        self.discriminant() == 7 || self.discriminant() == 8
    }

    /// Returns true if Item is an inline table.
    pub fn is_inline_table(&self) -> bool {
        self.discriminant() == 8
    }

    /// Returns true if Item is a string.
    pub fn is_string(&self) -> bool {
        self.discriminant() == 9
    }

    /// Returns true if Item is AOT.
    pub fn is_aot(&self) -> bool {
        self.discriminant() == 10
    }
}

        }
    }
}

impl<'a> Item<'a> {
    /// Returns true if Item is a value.
    pub fn is_value(&self) -> bool {
        match self.discriminant() {
            0 | 1 => false,
            _ => true,
        }
    }

    /// Returns true if Item is trivia.
    pub fn is_trivia(&self) -> bool {
        !self.is_value()
    }

    /// Returns true if Item is whitespace.
    pub fn is_ws(&self) -> bool {
        self.discriminant() == 0
    }

    /// Returns true if Item is a comment.
    pub fn is_comment(&self) -> bool {
        self.discriminant() == 1
    }

    /// Returns true if Item is an integer.
    pub fn is_integer(&self) -> bool {
        self.discriminant() == 2
    }

    /// Returns true if Item is a float.
    pub fn is_float(&self) -> bool {
        self.discriminant() == 3
    }

    /// Returns true if Item is a boolean.
    pub fn is_bool(&self) -> bool {
        self.discriminant() == 4
    }

    /// Returns true if Item is a date/time.
    pub fn is_date_time(&self) -> bool {
        self.discriminant() == 5
    }

    /// Returns true if Item is an array.
    pub fn is_array(&self) -> bool {
        self.discriminant() == 6
    }

    /// Returns true if Item is a table.
    pub fn is_table(&self) -> bool {
        self.discriminant() == 7 || self.discriminant() == 8
    }

    /// Returns true if Item is an inline table.
    pub fn is_inline_table(&self) -> bool {
        self.discriminant() == 8
    }

    /// Returns true if Item is a string.
    pub fn is_string(&self) -> bool {
        self.discriminant() == 9
    }

    /// Returns true if Item is AOT.
    pub fn is_aot(&self) -> bool {
        self.discriminant() == 10
    }
}
