use crate::error::{Error, InlineError};
use crate::lex::pattern::Pattern;
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

    fn parse<'a>(line: &'a str, store: &mut Store<'a, &'a str>) -> Result<Self, InlineError<'a>> {
        let delimiter = match line.find(':') {
            None => return Err(InlineError::expected(line.len(), ':')),
            Some(i) => i,
        };

        let (head, body) = line.split_at(delimiter);

        let body = Pattern::parse(body[1..].trim())?;

        let head = head.trim();
        if head.contains(|c: char| c.is_whitespace() || c == '*' || c == '+') {
            return Err(InlineError::invalid_rule_head(0, head));
        }
        let head = store.add(head);

        Ok(Self::new(head, body))
    }
}

#[derive(Serialize)]
pub struct Lexer(Vec<Rule>);

impl Lexer {
    pub fn parse<'a>(from: &'a str, store: &mut Store<'a, &'a str>) -> Result<Self, Error<'a>> {
        let mut rules = Vec::new();
        let mut i = 1;

        for line in from.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let line = match line.find('#') {
                None => &line,
                Some(i) => &line[..i],
            };

            let rule = Rule::parse(line, store).map_err(|e| e.lined(i))?;
            rules.push(rule);

            i += 1;
        }

        Ok(Self(rules))
    }
}
