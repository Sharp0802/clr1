use std::ops::RangeInclusive;

pub enum ClassItem {
    Char(char),
    Range(RangeInclusive<char>),
}

pub struct Class {
    pub deny: bool,
    pub list: &'static [ClassItem],
}

pub struct Quantifier {
    pub range: RangeInclusive<usize>,
    pub what: &'static Pattern,
}

pub enum Pattern {
    Reference(usize, Option<usize>),
    Literal(&'static str),
    Class(Class),
    Quantifier(Quantifier),
    Group(&'static [Pattern]),
    Or(&'static [Pattern]),
}
