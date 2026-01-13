use std::env;
use std::fs::write;
use std::path::Path;

fn main() {
    let base = env!("CARGO_MANIFEST_DIR");
    let lexer = format!("{}/../test.lex", base);
    let parser = format!("{}/../test.parse", base);

    println!("cargo:rerun-if-changed={}", lexer);
    println!("cargo:rerun-if-changed={}", parser);

    let dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not defined");
    let output = Path::new(&dir).join("clr1.rs");

    let generated = clr1_build::build(&lexer, &parser).expect("failed to generate parser");

    write(&output, &generated).expect("failed to write generated parser");
}
