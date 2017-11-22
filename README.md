# Molten

[![Build Status](https://travis-ci.org/LeopoldArkham/Molten.svg?branch=master)](https://travis-ci.org/LeopoldArkham/Molten)

## [WIP] Molten - Style-preserving TOML parser.

Molten is a WIP lossless [TOML](https://github.com/toml-lang/toml) parser that preserves all
comments, indentations, whitespace and internal element ordering, and makes all of these fully
editable via an easy API. It is written with the intent of replacing the current TOML parser
used in [cargo-edit](https://github.com/killercup/cargo-edit), and, eventually, adding that
functionality to Cargo itself.

### Goals
- Speed: Molten is a one-pass parser which avoids allocation.
- Unopinionated: Molten respects the way you wrote your document, to the letter.
- Fully addressable: All toml elements can be edited, created, or deleted via the API.

### Non-Goals
- Error recovery: Molten does not try to correct recoverable errors.
- Serde support: See [toml-rs](https://github.com/alexcrichton/toml-rs) for this.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Molten by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
