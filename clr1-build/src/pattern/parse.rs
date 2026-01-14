use std::ops::RangeInclusive;
use std::str::FromStr;
use crate::error::{Error, ErrorKind};
use crate::iter::{read_while, skip_while, Chars, Offset};
use crate::pattern::pattern::{Class, ClassItem, Pattern, Quantifier};
use crate::store::Store;

#[inline]
fn unreachable() -> ! {
    #[cfg(debug_assertions)]
    panic!("unreachable");
    #[cfg(not(debug_assertions))]
    unreachable!()
}

fn escape<'a>(from: &mut dyn Chars<'a>, begin: Offset) -> Result<char, Error> {
    fn nibble_to_int(b: char) -> Option<u8> {
        let c = if '0' <= b && b <= '9' {
            b as u8 - b'0'
        } else if 'a' <= b && b <= 'f' {
            b as u8 - b'a' + 10
        } else if 'A' <= b && b <= 'F' {
            b as u8 - b'A' + 10
        } else {
            return None;
        };

        Some(c)
    }

    let escaped = match from.next() {
        None => return Err(ErrorKind::UnclosedEscape.at(begin)),
        Some((_, 'x')) => {
            let Some((high_offset, high)) = from.next() else {
                return Err(ErrorKind::UnclosedEscape.at(begin));
            };

            let Some(high) = nibble_to_int(high) else {
                return Err(ErrorKind::InvalidEscape.at(high_offset));
            };

            let Some((low_offset, low)) = from.next() else {
                return Err(ErrorKind::UnclosedEscape.at(begin));
            };

            let Some(low) = nibble_to_int(low) else {
                return Err(ErrorKind::InvalidEscape.at(low_offset));
            };

            ((high << 4) | low) as char
        }
        Some((_, 'u')) => {
            match from.next() {
                Some((_, '{')) => {},
                _ => return Err(ErrorKind::InvalidEscape.at(begin)),
            };

            let (num_begin, _, str) = read_while(from, |ch: char| ch.is_ascii_digit())
                .ok_or_else(|| ErrorKind::UnclosedEscape.at(begin))?;

            let unicode = u32::from_str_radix(str, 16)
                .map_err(|_| ErrorKind::NumberTooBig.at(num_begin))?;

            match from.next() {
                Some((_, '}')) => {},
                _ => return Err(ErrorKind::UnclosedEscape.at(num_begin)),
            };

            char::from_u32(unicode)
                .ok_or_else(|| ErrorKind::InvalidUnicode.at(num_begin))?
        }
        Some((_, 'n')) => '\x0A',
        Some((_, 'r')) => '\x0D',
        Some((_, 't')) => '\x09',
        Some((_, 'v')) => '\x0B',
        Some((_, '\\')) => '\\',
        Some((_, '\'')) => '\'',
        Some((_, '{')) => '{',
        Some((_, '}')) => '}',
        Some((_, '[')) => '[',
        Some((_, ']')) => ']',
        Some((_, '(')) => '(',
        Some((_, ')')) => ')',
        Some((_, '|')) => '|',
        Some((_, '?')) => '?',
        Some((_, '*')) => '*',
        Some((_, '+')) => '+',
        Some((_, '"')) => '"',
        Some(_) => {
            return Err(ErrorKind::InvalidEscape.at(begin));
        }
    };

    Ok(escaped)
}

pub trait Parse<'a, T> {
    fn parse(from: &mut dyn Chars<'a>, begin: Offset, store: &mut Store<&'a str>) -> Result<T, Error>;
}

impl<'a> Parse<'a, Class> for Class {
    fn parse(from: &mut dyn Chars<'a>, begin: Offset, _: &mut Store<&'a str>) -> Result<Self, Error> {
        fn parse_item(from: &mut dyn Chars) -> Result<ClassItem, Error> {
            let (begin, ch) = match from.next() {
                // should be checked in Class::parse
                None | Some((_, ']')) => unreachable(),

                Some((offset, '\\')) => (offset, escape(from, offset)?),
                Some(pair) => pair,
            };

            Ok(if let Some((_, '-')) = from.peek() {
                from.next().unwrap();

                let end = match from.next() {
                    None | Some((_, ']')) => {
                        return Err(ErrorKind::UnclosedClassItem.at(begin));
                    }
                    Some((offset, '\\')) => escape(from, offset)?,
                    Some((_, ch)) => ch,
                };

                ClassItem::Range(ch..=end)
            } else {
                ClassItem::Char(ch)
            })
        }

        let deny = if let Some((_, '^')) = from.peek() {
            from.next().unwrap();
            true
        } else {
            false
        };

        let mut list = Vec::new();
        loop {
            match from.peek() {
                None => break Err(ErrorKind::UnclosedClass.at(begin)),
                Some((_, ']')) => {
                    from.next().unwrap();
                    break Ok(Class::new(deny, list));
                }
                Some(_) => {
                    list.push(parse_item(from)?);
                }
            }
        }
    }
}

impl<'a> Parse<'a, RangeInclusive<usize>> for Quantifier {
    fn parse(from: &mut dyn Chars<'a>, begin: Offset, _: &mut Store<&'a str>) -> Result<RangeInclusive<usize>, Error> {
        let (min, max_required) = match from.peek() {
            None => return Err(ErrorKind::UnclosedQuantifier.at(begin)),
            Some((_, ',')) => {
                from.next().unwrap();
                (0, true)
            }
            Some(_) => {
                let Some((begin, _, str)) = read_while(from, |ch: char| ch.is_ascii_digit())
                else {
                    return Err(ErrorKind::MalformedQuantifier.at(begin));
                };

                let min = usize::from_str(str)
                    .map_err(|_| ErrorKind::NumberTooBig.at(begin))?;

                (min, false)
            }
        };

        match from.next() {
            Some((_, ',')) => {
                let max = match read_while(from, |ch: char| ch.is_ascii_digit()) {
                    None => {
                        if max_required {
                            return Err(ErrorKind::MalformedQuantifier.at(begin));
                        } else {
                            usize::MAX
                        }
                    }
                    Some((begin, _, str)) => {
                        usize::from_str(str).map_err(|_| ErrorKind::NumberTooBig.at(begin))?
                    }
                };

                match from.next() {
                    Some((_, '}')) => Ok(min..=max),
                    Some((offset, _)) => Err(ErrorKind::MalformedQuantifier.at(offset)),
                    None => Err(ErrorKind::UnclosedQuantifier.at(begin)),
                }
            }
            Some((_, '}')) => {
                if max_required {
                    Err(ErrorKind::MalformedQuantifier.at(begin))
                } else {
                    Ok(min..=min)
                }
            }
            Some((offset, _)) => Err(ErrorKind::MalformedQuantifier.at(offset)),
            None => Err(ErrorKind::UnclosedQuantifier.at(begin))
        }
    }
}

impl<'a> Parse<'a, Vec<Pattern>> for Vec<Pattern> {
    fn parse(from: &mut dyn Chars<'a>, begin: Offset, store: &mut Store<&'a str>) -> Result<Vec<Pattern>, Error> {
        let mut items = Vec::new();
        loop {
            skip_while(from, is_whitespace);
            
            match from.peek() {
                None => break Err(ErrorKind::UnclosedGroup.at(begin)),
                Some((_, ')')) => {
                    from.next().unwrap();
                    break Ok(items);
                },
                Some(_) => {}
            };

            if !parse_once(from, &mut items, store)? {
                break Err(ErrorKind::UnclosedGroup.at(begin));
            };
        }
    }
}

impl<'a> Parse<'a, String> for String {
    fn parse(from: &mut dyn Chars<'a>, begin: Offset, _: &mut Store<&'a str>) -> Result<String, Error> {
        let mut buffer = String::new();
        loop {
            let ch = match from.next() {
                None => break Err(ErrorKind::UnclosedLiteral.at(begin)),
                Some((_, '\'')) => break Ok(buffer),

                Some((offset, '\\')) => escape(from, offset)?,
                Some((_, ch)) => ch
            };

            buffer.push(ch);
        }
    }
}


// `char.is_whitespace` doesn't check vertical tab (WhatWG).
// This follows POSIX locale
#[inline]
pub fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ('\t'..='\r').contains(&ch)
}

macro_rules! pop {
    ($from:expr, $offset:expr) => {
        $from
            .pop()
            .ok_or_else(|| ErrorKind::MissingPrefix.at($offset))
    };
}

fn parse_once<'a>(from: &mut dyn Chars<'a>, stack: &mut Vec<Pattern>, store: &mut Store<&'a str>) -> Result<bool, Error> {
    skip_while(from, is_whitespace);

    let str = from.as_str();
    let pat = match from.next() {
        None => return Err(ErrorKind::UnexpectedEOF.at(Offset::new(0, 0))),
        Some((_, ';')) => return Ok(false),

        Some((i, '\'')) => String::parse(from, i, store)?.into(),

        Some((_, '.')) => Class::new(true, vec![]).into(),
        Some((i, '[')) => Class::parse(from, i, store)?.into(),
        Some((i, '(')) => Vec::<Pattern>::parse(from, i, store).map(Pattern::Group)?,
        Some((i, '|')) => {
            let mut rhs = match stack.pop() {
                None => return Err(ErrorKind::MissingPrefix.at(i)),
                Some(Pattern::Or(rhs)) => rhs,
                Some(rhs) => vec![rhs],
            };

            let mut local = Vec::new();
            if !parse_once(from, &mut local, store)? {
                return Err(ErrorKind::MissingSuffix.at(i));
            };

            let lhs = local.pop().unwrap();

            rhs.push(lhs);

            Pattern::Or(rhs)
        }

        Some((i, '?')) => Quantifier::new(pop!(stack, i)?, 0..=1).into(),
        Some((i, '*')) => Quantifier::new(pop!(stack, i)?, 0..=usize::MAX).into(),
        Some((i, '+')) => Quantifier::new(pop!(stack, i)?, 1..=usize::MAX).into(),
        Some((i, '{')) => Quantifier::new(pop!(stack, i)?, Quantifier::parse(from, i, store)?).into(),

        Some((i, ')' | ']' | '}')) => {
            return Err(ErrorKind::CloserMismatched.at(i));
        }

        Some((i, ch)) => {
            let mut size = ch.len_utf8();
            let (named, offset) = loop {
                match from.next() {
                    None => break (false, i),
                    Some((offset, '@')) => break (true, offset),
                    Some((offset, ch)) => {
                        if ch.is_alphanumeric() || ch == '_' {
                            size += ch.len_utf8()
                        } else {
                            break (false, offset)
                        }
                    }
                }
            };

            let reference = store.add(&str[..size]);

            let name = if named {
                let Some((_, _, name)) = read_while(from, |ch: char| !is_whitespace(ch)) else {
                    return Err(ErrorKind::EmptyName.at(offset))
                };

                Some(store.add(name))
            } else {
                None
            };

            Pattern::Reference(reference, name)
        }
    };

    stack.push(pat);

    Ok(true)
}

pub fn parse<'a>(from: &mut dyn Chars<'a>, store: &mut Store<&'a str>) -> Result<Pattern, Error> {
    let mut list: Vec<Pattern> = Vec::new();
    while parse_once(from, &mut list, store)? {}
    Ok(Pattern::Group(list))
}
