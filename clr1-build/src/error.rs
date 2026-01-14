use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use crate::iter::Offset;
use crate::ser;

#[derive(Debug)]
pub struct Error {
    at: Offset,
    kind: ErrorKind,
}

impl Error {
    pub fn new(at: Offset, kind: ErrorKind) -> Self {
        Self { at, kind }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.at, self.kind)
    }
}

impl std::error::Error for Error {}

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization failed: {0}")]
    Serialization(#[from] ser::Error),
    #[error("expected '{0}'")]
    Expected(char),
    #[error("unexpected end of file")]
    UnexpectedEOF,
    #[error("unknown escape sequence")]
    InvalidEscape,
    #[error("unknown unicode")]
    InvalidUnicode,
    #[error("escape sequence not closed")]
    UnclosedEscape,
    #[error("character class not closed")]
    UnclosedClass,
    #[error("character class item not closed")]
    UnclosedClassItem,
    #[error("quantifier not closed")]
    UnclosedQuantifier,
    #[error("group not closed")]
    UnclosedGroup,
    #[error("literal not closed")]
    UnclosedLiteral,
    #[error("close character mismatched")]
    CloserMismatched,
    #[error("invalid quantifier sequence")]
    MalformedQuantifier,
    #[error("number exceeds limit")]
    NumberTooBig,
    #[error("prefix missing")]
    MissingPrefix,
    #[error("suffix missing")]
    MissingSuffix,
    #[error("invalid non-terminal name")]
    InvalidRuleHead,
    #[error("name cannot be empty")]
    EmptyName,
}

impl ErrorKind {
    pub fn at(self, at: Offset) -> Error {
        Error::new(at, self)
    }
}
