use std::env;
use std::fs::write;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lexer = base.join("../test.lex").canonicalize().unwrap();
    let parser = base.join("../test.parse").canonicalize().unwrap();

    println!("cargo:rerun-if-changed={}", lexer.display());
    println!("cargo:rerun-if-changed={}", parser.display());

    let dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not defined");
    let output = Path::new(&dir).join("clr1.rs");

    let generated = match clr1_build::build(&lexer, &parser) {
        Ok(generated) => generated,
        Err(e) => {
            eprintln!("{}{}", lexer.display(), e);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = write(&output, &generated) {
        eprintln!("{}: {}", output.display(), e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
