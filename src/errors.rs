error_chain! {
    errors {

        /// This error occurs when the parser encounters a syntax error
        /// in the TOML being parsed. The error references the line and
        /// location within the line where the error was encountered.
        ParseError(line: usize, col: usize) {
            description("TOML parse error")
            display("TOML parse error line {} column {}", line, col)
        }

        /// A duplicate key was found.
        DuplicateKey(k: String) {
            description("Duplicate key")
            display("Duplicate key: {}", k)
        }

        /// An array was found that had two or more element types.
        MixedArrayTypes {
            description("Mixed types found in array")
        }

        /// A numeric or date field was improperly specified.
        InvalidNumberOrDate {
            description("Invalid number or date format")
        }

        /// An unexpected character was found during parsing.
        UnexpectedChar(ch: char) {
            description("Unexpected character")
            display("Unexpected character: {}", ch)
        }

        /// The TOML being parsed ended before the end of a statement.
        UnexpectedEof {
            description("Unepxected end of file")
        }

        /// The string being parsed contains an invalid character.
        InvalidCharInString(ch: char) {
            description("Invalid character in string")
            display("Invalid character '{}' in string", ch) // XXX escaping
        }

        /// An error that indicates a bug in the parser.
        InternalParserError(msg: String) {
            description("Internal Parser Error")
            display("{}\n The parser has violated one of its invariants.
            This is a bug.\n
            Please open an issue uqoting this message at
            https://github.com/LeopoldArkham/Molten", msg)
        }
    }
}
