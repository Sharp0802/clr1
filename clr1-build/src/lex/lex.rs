use crate::error::{Error, ErrorKind};
use crate::iter::{read_while, skip_while, Chars, Iter, Offset};
use crate::pattern::{is_whitespace, parse, Pattern};
use crate::store::Store;
use serde::Serialize;

#[derive(Serialize)]
pub struct Rule {
    id: usize,
    pat: Pattern,
}

impl Rule {
    fn new(id: usize, pat: Pattern) -> Self {
        Self { id, pat }
    }
}

#[derive(Serialize)]
pub struct Lexer(Vec<Rule>);

impl Lexer {
    pub fn parse<'a>(from: &'a str, store: &mut Store<&'a str>) -> Result<Self, Error> {
        let mut from: Iter<'a> = from.into();

        let mut rules = Vec::new();
        loop {
            skip_while(&mut from, is_whitespace);

            match from.peek() {
                Some((_, '#')) => {
                    from.next().unwrap();
                    skip_while(&mut from, |ch: char| ch != '\n');
                    continue;
                }
                None => break Ok(Self(rules)),
                _ => {}
            }

            let Some((begin, end, name)) =
                read_while(&mut from, |ch: char| !is_whitespace(ch) && ch != ':')
            else {
                // starts with ':'
                let (offset, _) = from.peek().unwrap();
                break Err(ErrorKind::EmptyName.at(offset));
            };
            if name.contains([':', '@']) {
                break Err(ErrorKind::InvalidRuleHead.at(begin));
            }
            let name = store.add(name);

            skip_while(&mut from, is_whitespace);

            // consume ':'
            match from.next() {
                Some((_, ':')) => {}
                Some((offset, _)) => break Err(ErrorKind::Expected(':').at(offset)),
                None => {
                    break Err(
                        ErrorKind::Expected(':').at(Offset::new(end.line(), end.column() + 1))
                    );
                }
            }

            let pattern = parse(&mut from, store)?;

            rules.push(Rule::new(name, pattern));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ser;
    use super::*;

    #[test]
    fn test() {
        let mut store = Store::new();
        let lexer = Lexer::parse(
            r#"
# Basic character classes and quantifiers
Digit : [0-9] ;
Hex   : [0-9a-fA-F] ;

# Testing the fixed quantifier logic {n,m}
# This matches an IPv4 address (e.g., 192.168.1.1)
IPv4 : ([0-9]{1,3} '.'){3} [0-9]{1,3} ;

# Testing references to other rules and alternation
# Matches '0x' followed by hex digits OR just digits
Number : '0x' Hex+ | Digit+ ;

# Testing grouping and whitespace
# Matches a simple variable assignment like "x = 10"
Assignment : [a-z]+ [ \t]* '=' [ \t]* Number ;

# Testing string literals with escaped quotes
# Matches "hello \"world\""
String : '"' ( [^"\\\n] | ('\\' .) )* '"' ;
            "#,
            &mut store,
        )
        .unwrap();
        
        println!("{}", ser::to_string(&lexer, Default::default()).unwrap());
    }
}
