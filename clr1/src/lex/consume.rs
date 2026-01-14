use std::ops::RangeInclusive;
use crate::lex::pattern::{Class, ClassItem, Pattern, Quantifier};

pub trait Consume {
    fn consume(&self, from: &str) -> Option<usize>;
}

impl Consume for char {
    fn consume(&self, from: &str) -> Option<usize> {
        if from.chars().next() == Some(*self) {
            Some(self.len_utf8())
        } else {
            None
        }
    }
}

impl Consume for str {
    fn consume(&self, from: &str) -> Option<usize> {
        if from.starts_with(self) {
            Some(self.len())
        } else {
            None
        }
    }
}

impl Consume for RangeInclusive<char> {
    fn consume(&self, from: &str) -> Option<usize> {
        let Some(ch) = from.chars().next() else {
            return None;
        };

        if self.contains(&ch) {
            Some(ch.len_utf8())
        } else {
            None
        }
    }
}

impl Consume for ClassItem {
    fn consume(&self, from: &str) -> Option<usize> {
        match self {
            ClassItem::Char(ch) => ch.consume(from),
            ClassItem::Range(range) => range.consume(from),
        }
    }
}

impl Consume for Class {
    fn consume(&self, from: &str) -> Option<usize> {
        if self.deny {
            for item in self.list {
                if item.consume(from).is_some() {
                    return None;
                }
            }

            from.chars().next().map(|ch| ch.len_utf8()).or(Some(0))
        } else {
            for item in self.list {
                if let Some(size) = item.consume(from) {
                    return Some(size);
                }
            }

            None
        }
    }
}

impl Consume for Quantifier {
    fn consume(&self, from: &str) -> Option<usize> {
        let mut total = 0;

        for _ in 0..*self.range.start() {
            if let Some(size) = self.what.consume(&from[total..]) {
                total += size;
            } else {
                return None;
            }
        }

        for _ in *self.range.start()..*self.range.end() {
            if let Some(size) = self.what.consume(&from[total..]) {
                total += size;
            } else {
                break;
            }
        }

        Some(total)
    }
}

impl Consume for [Pattern] {
    fn consume(&self, from: &str) -> Option<usize> {
        let mut total = 0;

        for pat in self {
            if let Some(size) = pat.consume(&from[total..]) {
                total += size;
            } else {
                return None;
            }
        }

        Some(total)
    }
}

impl Consume for Pattern {
    fn consume(&self, from: &str) -> Option<usize> {
        match self {
            Pattern::Reference(_, _) => None,
            Pattern::Literal(str) => str.consume(from),
            Pattern::Class(class) => class.consume(from),
            Pattern::Quantifier(quantifier) => quantifier.consume(from),
            Pattern::Group(group) => group.consume(from),
            Pattern::Or(list) => {
                for pat in *list {
                    if let Some(size) = pat.consume(from) {
                        return Some(size);
                    }
                }

                None
            }
        }
    }
}
