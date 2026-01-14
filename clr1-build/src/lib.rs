use crate::error::{Error, ErrorKind};
use crate::iter::Offset;
use crate::lex::Lexer;
use crate::ser::Options;
use crate::store::Store;
use std::fs::read_to_string;
use std::path::Path;

mod error;
mod lex;
mod store;

mod iter;
mod parse;
mod pattern;
mod ser;
mod util;

pub fn build(lexer: impl AsRef<Path>, parser: impl AsRef<Path>) -> Result<String, Error> {
    let mut store = Store::new();

    let lex = read_to_string(lexer).map_err(|e| ErrorKind::Io(e).at(Offset::new(0, 0)))?;
    let lexer = Lexer::parse(&lex, &mut store)?;

    let lexer = ser::to_string(&lexer, Options { initial_indent: 2 })
        .map_err(|e| ErrorKind::from(e).at(Offset::new(0, 0)))?;

    let generated = format!(
        r#"
mod generated {{
    use clr1::lex::*;

    static LEXER: Lexer = {};
}}
"#,
        lexer
    );

    Ok(generated)
}
