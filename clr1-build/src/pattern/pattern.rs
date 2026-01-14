use crate::util::Boxed;
use derive_more::with_trait::From;
use serde::Serialize;
use std::ops::RangeInclusive;

// [LIST]
// [^LIST]
#[derive(Serialize)]
pub struct Class {
    deny: bool,
    list: Vec<ClassItem>,
}

impl Class {
    pub fn new(deny: bool, list: Vec<ClassItem>) -> Self {
        Self { deny, list }
    }
}

#[derive(Serialize)]
pub enum ClassItem {
    Char(char),
    Range(RangeInclusive<char>),
}

// PAT?
// PAT*
// PAT+
// PAT{n,m?}
// PAT{,m}
#[derive(Serialize)]
pub struct Quantifier {
    range: RangeInclusive<usize>,
    what: Boxed<Pattern>,
}

impl Quantifier {
    pub fn new(pat: Pattern, range: RangeInclusive<usize>) -> Self {
        Self {
            range,
            what: Boxed::new(pat),
        }
    }
}

#[derive(Serialize, From)]
pub enum Pattern {
    Reference(usize, Option<usize>),
    Literal(String),
    Class(Class),
    Quantifier(Quantifier),
    #[from(skip)]
    Group(Vec<Pattern>),
    #[from(skip)]
    Or(Vec<Pattern>),
}
