use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

pub(crate) struct InlineError<'a> {
    offset: usize,
    kind: ErrorKind<'a>,
}

impl Display for InlineError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.offset, self.kind)
    }
}

impl Debug for InlineError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl std::error::Error for InlineError<'_> {}

impl<'a> InlineError<'a> {
    pub(crate) fn lined(self, line: usize) -> Error<'a> {
        Error::new(line, self.offset, self.kind)
    }
}

pub struct Error<'a> {
    line: usize,
    offset: usize,
    kind: ErrorKind<'a>,
}

impl<'a> Error<'a> {
    pub(crate) fn new(line: usize, offset: usize, kind: ErrorKind<'a>) -> Self {
        Self { line, offset, kind }
    }
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}:{}", self.line, self.offset, self.kind)
    }
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl std::error::Error for Error<'_> {}

#[derive(Error, Debug)]
pub enum ErrorKind<'a> {
    #[error("expected '{0}'")]
    Expected(char),
    #[error("Unknown escape sequence '\\{0}'")]
    InvalidEscape(&'a str),
    #[error("escape sequence not closed")]
    UnclosedEscape,
    #[error("character class not closed")]
    UnclosedClass,
    #[error("quantifier not closed")]
    UnclosedQuantifier,
    #[error("group not closed")]
    UnclosedGroup,
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
    #[error("Invalid non-terminal name '{0}'")]
    InvalidRuleHead(&'a str),
}

impl<'a> InlineError<'a> {
    pub fn expected(offset: usize, expected: char) -> Self {
        Self {
            offset,
            kind: ErrorKind::Expected(expected),
        }
    }

    pub fn invalid_escape(offset: usize, seq: &'a str) -> Self {
        Self {
            offset,
            kind: ErrorKind::InvalidEscape(seq),
        }
    }

    pub fn unclosed_escape(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::UnclosedEscape,
        }
    }

    pub fn unclosed_class(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::UnclosedClass,
        }
    }

    pub fn unclosed_quantifier(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::UnclosedQuantifier,
        }
    }

    pub fn unclosed_group(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::UnclosedGroup,
        }
    }

    pub fn closer_mismatched(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::CloserMismatched,
        }
    }

    pub fn malformed_quantifier(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::MalformedQuantifier,
        }
    }

    pub fn number_too_big(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::NumberTooBig,
        }
    }

    pub fn missing_prefix(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::MissingPrefix,
        }
    }

    pub fn missing_suffix(offset: usize) -> Self {
        Self {
            offset,
            kind: ErrorKind::MissingSuffix,
        }
    }

    pub fn invalid_rule_head(offset: usize, name: &'a str) -> Self {
        Self {
            offset,
            kind: ErrorKind::InvalidRuleHead(name),
        }
    }
}
