use crate::lex::Lexer;
use crate::ser::Options;
use crate::store::Store;
use std::fs::read_to_string;

pub mod error;
pub mod lex;
pub mod store;

mod ser;

pub fn build(lexer: &str, parser: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut store = Store::new();

    let lex = read_to_string(lexer)?;
    let lexer = Lexer::parse(&lex, &mut store).map_err(|err| err.to_string())?;

    let lexer = ser::to_string(&lexer, Options {
        initial_indent: 2
    })?;

    let generated = format!(r#"
mod generated {{
    use clr1::lex::*;

    static LEXER: Lexer = {};
}}
"#, lexer);

    Ok(generated)
}
