#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate Molten;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = ::std::str::from_utf8(data) {
        let _ = Molten::parser::Parser::new(text).parse();
    }
});
