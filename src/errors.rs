//! Error types for Molten

#![allow(missing_docs)]
error_chain! {

    foreign_links {
        ParseIntError(::std::num::ParseIntError);
        ParseFloatError(::std::num::ParseFloatError);
        ParseBoolError(::std::str::ParseBoolError);
        ParseDateTimeError(::chrono::ParseError);
        ReadFileError(::std::io::Error);
    }

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

        /// todo: parametrize
        APIWrongItem {
            description("Function not implemented for this Item variant.")
        }

        /// An error that indicates a bug in the parser.
        InternalParserError(msg: String) {
            description("Internal Parser Error")
            display("{}\n The parser has violated one of its invariants.
            This is a bug.\n
            Please open an issue citing this message at
            https://github.com/LeopoldArkham/Molten", msg)
        }

        /// A non-existent key was used
        NonExistentKey(key: String) {
            description("The requested key does not exist in this container.")
            display("Key \"{}\"does not exist.", key)
        }

        /// A string was found that did not match required string syntax.
        ParseStringError {
            description("Input does not contain a valid string.")
        }
    }
}
