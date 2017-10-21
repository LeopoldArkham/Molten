error_chain! {
    errors {
        ParseError(line: usize, col: usize) {
            description("TOML parse error")
            display("TOML parse error line {} column {}", line, col)
        }
        DuplicateKey(k: String) {
            description("Duplicate key")
            display("Duplicate key: {}", k)
        }
        MixedArrayTypes {
            description("Mixed types found in array")
        }
        InvalidNumberOrDate {
            description("Invalid number or date format")
        }
        UnexpectedChar(ch: char) {
            description("Unexpected character")
            display("Unexpected character: {}", ch)
        }
    }
}
