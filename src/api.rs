use items::*;

impl<'a> Item<'a> {

    pub fn is_value(&self) -> bool {
        match self.discriminant() {
            0 | 1 => false,
            _ => true,
        }
    }

    pub fn is_trivia(&self) -> bool {
        !self.is_value()
    }

    pub fn is_ws(&self) -> bool {
        self.discriminant() == 0
    }

    pub fn is_comment(&self) -> bool {
        self.discriminant() == 1
    }

    pub fn is_integer(&self) -> bool {
        self.discriminant() == 2
    }

    pub fn is_float(&self) -> bool {
        self.discriminant() == 3
    }

    pub fn is_bool(&self) -> bool {
        self.discriminant() == 4
    }

    pub fn is_date_time(&self) -> bool {
        self.discriminant() == 5
    }

    pub fn is_array(&self) -> bool {
        self.discriminant() == 6
    }

    pub fn is_table(&self) -> bool {
        self.discriminant() == 7 || self.discriminant() == 8
    }

    pub(crate) fn is_real_table(&self) -> bool {
        self.discriminant() == 7 || self.discriminant() == 10
    }

    pub fn is_inline_table(&self) -> bool {
        self.discriminant() == 8
    }

    pub fn is_string(&self) -> bool {
        self.discriminant() == 9
    }

    pub fn is_aot(&self) -> bool {
        self.discriminant() == 10
    }

}
