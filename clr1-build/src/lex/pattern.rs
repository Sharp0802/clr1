use crate::error::InlineError;
use crate::lex::iter::Iter;
use std::ops::RangeInclusive;
use std::str::FromStr;
use serde::Serialize;

#[derive(Serialize)]
pub enum ClassItem {
    Char(char),
    Range(RangeInclusive<char>),
}

// [LIST]
// [^LIST]
#[derive(Serialize)]
pub struct Class {
    deny: bool,
    list: Vec<ClassItem>,
}

impl Class {
    fn new(deny: bool, list: Vec<ClassItem>) -> Self {
        Self { deny, list }
    }
}

#[derive(Serialize)]
struct Boxed<T>(Box<T>);

impl<T> Boxed<T> {
    fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

// PAT?
// PAT*
// PAT+
// PAT{n?,m?}
#[derive(Serialize)]
pub struct Quantifier {
    range: RangeInclusive<usize>,
    what: Boxed<Pattern>,
}

impl Quantifier {
    fn new(pat: Pattern, range: RangeInclusive<usize>) -> Self {
        Self {
            range,
            what: Boxed::new(pat),
        }
    }

    fn optional(pat: Pattern) -> Self {
        Self {
            range: 0..=1,
            what: Boxed::new(pat),
        }
    }

    fn at_least(pat: Pattern, min: usize) -> Self {
        Self {
            range: min..=usize::MAX,
            what: Boxed::new(pat),
        }
    }

    fn at_most(pat: Pattern, max: usize) -> Self {
        Self {
            range: 0..=max,
            what: Boxed::new(pat),
        }
    }

    fn exactly(pat: Pattern, count: usize) -> Self {
        Self {
            range: count..=count,
            what: Boxed::new(pat),
        }
    }
}

#[derive(Serialize)]
pub enum Pattern {
    Char(char),
    Class(Class),
    Quantifier(Quantifier),
    Group(Vec<Pattern>),
    Or(Vec<Pattern>),
}

impl Pattern {
    fn nibble(b: char) -> Option<u8> {
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

    fn escape<'a>(from: &mut Iter<'a>, begin: &'a str) -> Result<char, InlineError<'a>> {
        let escaped = match from.next() {
            None => return Err(InlineError::unclosed_escape(from.offset())),
            Some('x') => {
                let Some(high) = from.next() else {
                    return Err(InlineError::unclosed_escape(from.offset()));
                };

                let count = 2 /* \x */ + high.len_utf8();
                let Some(high) = Self::nibble(high) else {
                    return Err(InlineError::invalid_escape(from.offset(), &begin[..count]));
                };

                let Some(low) = from.next() else {
                    return Err(InlineError::unclosed_escape(from.offset()));
                };

                let count = count + low.len_utf8();
                let Some(low) = Self::nibble(low) else {
                    return Err(InlineError::invalid_escape(from.offset(), &begin[..count]));
                };

                ((high << 4) | low) as char
            }
            Some('u') => {
                let offset = from.offset();

                match from.next() {
                    None => return Err(InlineError::invalid_escape(offset, &begin[..2])),
                    Some('{') => {}
                    Some(_) => return Err(InlineError::invalid_escape(offset, &begin[..3])),
                }

                let mut unicode: u32 = 0;
                let mut count: usize = 3;
                loop {
                    let Some(nibble) = from.peek() else {
                        return Err(InlineError::unclosed_escape(offset));
                    };

                    if let Some(code) = Self::nibble(nibble) {
                        from.next().unwrap();

                        unicode <<= 4;
                        unicode |= code as u32;
                        count += nibble.len_utf8();
                    } else if nibble == '}' {
                        from.next().unwrap();
                        break;
                    } else {
                        count += nibble.len_utf8();
                        return Err(InlineError::invalid_escape(offset, &begin[..count]));
                    }

                    if unicode >= 0x0FFFFFFF {
                        return Err(InlineError::number_too_big(offset));
                    }
                }
                if count == 3 {
                    return Err(InlineError::invalid_escape(offset, &begin[..count]));
                }

                match char::from_u32(unicode) {
                    None => return Err(InlineError::invalid_escape(offset, &begin[..count])),
                    Some(ch) => ch,
                }
            }
            Some('n') => '\x0A',
            Some('r') => '\x0D',
            Some('t') => '\x09',
            Some('v') => '\x0B',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some('{') => '{',
            Some('}') => '}',
            Some('[') => '[',
            Some(']') => ']',
            Some('(') => '(',
            Some(')') => ')',
            Some('|') => '|',
            Some('?') => '?',
            Some('*') => '*',
            Some('+') => '+',
            Some('"') => '"',
            Some(ch) => {
                return Err(InlineError::invalid_escape(
                    from.offset(),
                    &begin[..1 + ch.len_utf8()],
                ));
            }
        };

        Ok(escaped)
    }

    fn parse_class<'a>(from: &mut Iter<'a>) -> Result<Pattern, InlineError<'a>> {
        let offset = from.offset() - 1;

        let mut first_offset = from.offset();
        let Some(mut first) = from.next() else {
            return Err(InlineError::unclosed_class(first_offset));
        };

        let deny = first == '^';
        if deny {
            first_offset = from.offset();
            first = from
                .next()
                .ok_or_else(|| InlineError::unclosed_class(first_offset))?;
        }

        if first == ']' {
            return Ok(Pattern::Class(Class::new(deny, Vec::new())));
        }

        let mut class: Vec<(ClassItem, usize)> = vec![(ClassItem::Char(first), first_offset)];
        let mut range_min: Option<(char, usize)> = None;

        loop {
            let begin = from.as_str();
            let item_offset = from.offset();

            match from.next() {
                None => return Err(InlineError::unclosed_class(offset)),
                Some(']') => {
                    if let Some((_, range_offset)) = range_min {
                        return Err(InlineError::missing_suffix(range_offset));
                    }

                    break;
                }
                Some('-') => {
                    if let Some((_, range_offset)) = range_min {
                        return Err(InlineError::missing_suffix(range_offset));
                    }

                    match class.pop() {
                        None | Some((ClassItem::Range(_), _)) => {
                            return Err(InlineError::missing_prefix(item_offset));
                        }
                        Some((ClassItem::Char(ch), offset)) => {
                            range_min = Some((ch, offset));
                        }
                    };
                }
                Some('\\') => {
                    let escaped = Self::escape(from, begin)?;
                    if let Some((min, offset)) = range_min {
                        class.push((ClassItem::Range(min..=escaped), offset));
                        range_min = None;
                    } else {
                        class.push((ClassItem::Char(escaped), item_offset));
                    }
                }
                Some(ch) => {
                    if let Some((min, offset)) = range_min {
                        class.push((ClassItem::Range(min..=ch), offset));
                        range_min = None;
                    } else {
                        class.push((ClassItem::Char(ch), item_offset));
                    }
                }
            };
        }

        Ok(Pattern::Class(Class::new(
            deny,
            class.into_iter().map(|(item, _)| item).collect(),
        )))
    }

    fn parse_group<'a>(from: &mut Iter<'a>) -> Result<Pattern, InlineError<'a>> {
        let offset = from.offset() - 1;

        let mut local: Vec<Pattern> = Vec::new();
        loop {
            match from.peek() {
                None => {
                    return Err(InlineError::unclosed_group(offset));
                }
                Some(')') => {
                    from.next().unwrap();
                    break;
                }
                Some(_) => {}
            }

            Self::parse_once(from, &mut local)?;
        }

        Ok(Pattern::Group(local))
    }

    fn parse_quantifier<'a>(from: &mut Iter<'a>, previous: Pattern) -> Result<Pattern, InlineError<'a>> {
        let offset = from.offset() - 1;

        let begin = from.as_str();

        let Some(next) = from.next() else {
            return Err(InlineError::unclosed_quantifier(offset));
        };

        let number_offset = from.offset();

        let mut minimum: Option<usize> = None;
        if next.is_ascii_digit() {
            let mut count = 1;
            loop {
                let Some(ch) = from.next() else {
                    return Err(InlineError::unclosed_quantifier(offset));
                };

                if !ch.is_ascii_digit() {
                    if ch == ',' {
                        break;
                    } else if ch == '}' {
                        let value = usize::from_str(&begin[..count])
                            .map_err(|_| InlineError::number_too_big(number_offset))?;
                        return Ok(Pattern::Quantifier(Quantifier::exactly(previous, value)));
                    } else {
                        return Err(InlineError::unclosed_quantifier(offset));
                    }
                }

                count += 1;
            }

            let value = usize::from_str(&begin[..count])
                .map_err(|_| InlineError::number_too_big(number_offset))?;
            minimum = Some(value);
        } else if next != ',' {
            return Err(InlineError::unclosed_quantifier(offset));
        }

        let begin = from.as_str();
        let number_offset = from.offset();

        let Some(next) = from.next() else {
            return Err(InlineError::unclosed_quantifier(offset));
        };

        if next == '}' {
            return if let Some(minimum) = minimum {
                Ok(Pattern::Quantifier(Quantifier::at_least(previous, minimum)))
            } else {
                Err(InlineError::malformed_quantifier(offset))
            };
        } else if !next.is_ascii_digit() {
            return Err(InlineError::unclosed_quantifier(offset));
        }

        let mut count = 1;
        loop {
            let Some(ch) = from.next() else {
                return Err(InlineError::unclosed_quantifier(offset));
            };

            if !ch.is_ascii_digit() {
                if ch == '}' {
                    break;
                } else {
                    return Err(InlineError::unclosed_quantifier(offset));
                }
            }

            count += 1;
        }

        let maximum =
            usize::from_str(&begin[..count]).map_err(|_| InlineError::number_too_big(number_offset))?;

        let quantifier = if let Some(minimum) = minimum {
            Quantifier::new(previous, minimum..=maximum)
        } else {
            Quantifier::at_most(previous, maximum)
        };

        Ok(Pattern::Quantifier(quantifier))
    }

    fn parse_once<'a>(from: &mut Iter<'a>, stack: &mut Vec<Pattern>) -> Result<bool, InlineError<'a>> {
        let begin = from.as_str();
        let offset = from.offset();

        let pat = match from.next() {
            None => {
                return Ok(false);
            }
            Some('[') => Self::parse_class(from)?,
            Some('(') => Self::parse_group(from)?,
            Some('\\') => Self::escape(from, begin).map(Pattern::Char)?,
            Some('?') => {
                let Some(pat) = stack.pop() else {
                    return Err(InlineError::missing_prefix(offset));
                };

                Pattern::Quantifier(Quantifier::optional(pat))
            }
            Some('*') => {
                let Some(pat) = stack.pop() else {
                    return Err(InlineError::missing_prefix(offset));
                };

                Pattern::Quantifier(Quantifier::at_least(pat, 0))
            }
            Some('+') => {
                let Some(pat) = stack.pop() else {
                    return Err(InlineError::missing_prefix(offset));
                };

                Pattern::Quantifier(Quantifier::at_least(pat, 1))
            }
            Some('{') => {
                let Some(pat) = stack.pop() else {
                    return Err(InlineError::missing_prefix(offset));
                };

                Self::parse_quantifier(from, pat)?
            }
            Some('|') => {
                let mut rhs = match stack.pop() {
                    None => return Err(InlineError::missing_prefix(offset)),
                    Some(Pattern::Or(rhs)) => rhs,
                    Some(rhs) => vec![rhs],
                };

                let mut local = Vec::new();
                if !Self::parse_once(from, &mut local)? {
                    return Err(InlineError::missing_suffix(offset));
                };

                let lhs = local.pop().unwrap();

                rhs.push(lhs);

                Pattern::Or(rhs)
            }
            Some(')') | Some(']') | Some('}') => {
                return Err(InlineError::closer_mismatched(offset));
            }
            Some(ch) => Pattern::Char(ch),
        };

        stack.push(pat);

        Ok(true)
    }

    pub fn parse(from: &'_ str) -> Result<Pattern, InlineError<'_>> {
        let mut iter = Iter::new(from);
        let mut list: Vec<Pattern> = Vec::new();
        while Self::parse_once(&mut iter, &mut list)? {}
        Ok(Pattern::Group(list))
    }
}
