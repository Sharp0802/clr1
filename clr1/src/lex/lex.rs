use crate::lex::consume::Consume;
use crate::lex::pattern::Pattern;

pub struct Rule {
    pub id: usize,
    pub pat: Pattern,
}

pub struct Token<'a> {
    kind: usize,
    value: &'a str,
}

impl<'a> Token<'a> {
    fn new(kind: usize, value: &'a str) -> Self {
        Self { kind, value }
    }

    pub fn kind(&self) -> usize {
        self.kind
    }

    pub fn value(&self) -> &str {
        self.value
    }
}

pub struct Lexer(pub &'static [Rule]);

impl Lexer {
    pub fn lex_once<'a>(&self, from: &'a str) -> Option<Token<'a>> {
        let mut token: Option<Token> = None;
        for rule in self.0 {
            if let Some(size) = rule.pat.consume(from) {
                let previous = match &token {
                    None => 0,
                    Some(t) => t.value.len()
                };

                if previous < size {
                    token = Some(Token::new(rule.id, &from[..size]));
                }
            }
        }

        token
    }

    pub fn lex<'a>(&self, from: &'a str) -> Result<Vec<Token<'a>>, usize> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut offset: usize = 0;

        while offset < from.len() {
            let token = match self.lex_once(&from[offset..]) {
                None => return Err(offset),
                Some(token) => token,
            };

            offset += token.value.len();
            tokens.push(token);
        }
        
        Ok(tokens)
    }
}
