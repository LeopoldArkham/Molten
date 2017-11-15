# Molten
## [WIP] Molten - Style-preserving TOML parser.

Molten is a WIP lossless TOML parser that preserves all comments, indentations, whitespace and internal element ordering, and makes  all of these fully editable via an easy API.
It is written with the intent of replacing the current toml parser used in cargo-edit, and, eventually, adding that functionality to cargo itself.

### Goals
- Speed: Molten is a one-pass parser which avoids allocation.
- Unopinionated: Molten respects the way you wrote your document, to the letter.
- Fully addressable: All toml elements can be edited, created, or deleted via the API.

### Non-Goals
- Error recovery: Molten does not try to correct recoverable errors.
- Serialization/Deserialization: See [toml-rs](https://github.com/alexcrichton/toml-rs) for this.
