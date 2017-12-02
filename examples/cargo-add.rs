extern crate Molten;

use std::io::{Read, Write};
use std::fs::File;
use std::error::Error;

use Molten::{key_value};

fn  main() {
    match run() {
        Err(e) => println!("{}", e),
        _ => {}
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut buf = String::new();
    let mut  f = File::open("examples/_cargo.toml")?;
    f.read_to_string(&mut buf)?;
    let mut manifest = Molten::parser::Parser::new(&buf).parse()?;


    let new_dep = key_value("parsehole = \"6.2.8\"")?;
    manifest["dependencies"].append(new_dep.0, new_dep.1)?;

    let mut out = File::create("examples/cargo_new.toml")?;
    out.write(manifest.as_string().as_bytes())?;

    Ok(())
}