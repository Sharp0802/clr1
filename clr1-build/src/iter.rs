use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Offset(usize, usize);

impl Offset {
    pub fn new(line: usize, column: usize) -> Self {
        Self(line, column)
    }

    pub fn line(&self) -> usize {
        self.0
    }

    pub fn column(&self) -> usize {
        self.1
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0 + 1, self.1 + 1)
    }
}

pub trait Chars<'a> : Iterator<Item = (Offset, char)> {
    fn peek(&self) -> Option<(Offset, char)>;
    fn as_str(&self) -> &'a str;
}

pub trait Pred {
    fn contains(&self, ch: char) -> bool;
}

impl Pred for char {
    fn contains(&self, ch: char) -> bool {
        ch == *self
    }
}

impl Pred for RangeInclusive<char> {
    fn contains(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

impl Pred for [char] {
    fn contains(&self, ch: char) -> bool {
        self.iter().any(|&item| item == ch)
    }
}

impl<T: Fn(char) -> bool> Pred for T {
    fn contains(&self, ch: char) -> bool {
        self(ch)
    }
}

pub fn read_while<'a>(chars: &mut dyn Chars<'a>, pred: impl Pred) -> Option<(Offset, Offset, &'a str)> {
    let begin = if let Some((offset, ch)) = chars.peek() {
        if !pred.contains(ch) {
            return None;
        }

        offset
    } else {
        return None;
    };

    let s = chars.as_str();

    let mut size = 0;
    let mut last = begin;
    loop {
        match chars.peek() {
            None => {
                break;
            }
            Some((offset, ch)) => {
                if !pred.contains(ch) {
                    break;
                }

                chars.next().unwrap();

                last = offset;
                size += ch.len_utf8();
            }
        }
    }

    Some((begin, last, &s[..size]))
}

pub fn skip_while<'a>(chars: &mut dyn Chars<'a>, pred: impl Pred) {
    loop {
        let ch = match chars.peek() {
            None => break,
            Some((_, ch)) => ch
        };

        if !pred.contains(ch) {
            break;
        }

        chars.next().unwrap();
    }
}

pub struct Iter<'a> {
    str: &'a str,
    i: usize,
    offset: Offset,
}

impl<'a> Chars<'a> for Iter<'a> {
    fn peek(&self) -> Option<(Offset, char)> {
        self.str[self.i..]
            .chars()
            .next()
            .map(|ch| (self.offset, ch))
    }

    fn as_str(&self) -> &'a str {
        &self.str[self.i..]
    }
}

impl Iterator for Iter<'_> {
    type Item = (Offset, char);

    fn next(&mut self) -> Option<Self::Item> {
        match self.str[self.i..].chars().next() {
            None => None,
            Some('\r') => {
                // ignore
                self.i += 1;
                self.next()
            }
            Some(ch) => {
                self.i += ch.len_utf8();

                let offset = self.offset;
                if ch == '\n' {
                    self.offset.0 += 1;
                    self.offset.1 = 0;
                } else {
                    self.offset.1 += 1;
                }

                Some((offset, ch))
            }
        }
    }
}

impl<'a> From<&'a str> for Iter<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            str: value,
            i: 0,
            offset: Offset(0, 0)
        }
    }
}
