
pub(crate) struct Iter<'a> {
    str: &'a str,
    i: usize,
}

impl Iterator for Iter<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peek() {
            None => None,
            Some(ch) => {
                self.i += ch.len_utf8();
                Some(ch)
            }
        }
    }
}

impl<'a> Iter<'a> {
    pub(crate) fn new(str: &'a str) -> Self {
        Self { str, i: 0 }
    }

    pub(crate) fn as_str(&self) -> &'a str {
        &self.str[self.i..]
    }

    pub(crate) fn peek(&self) -> Option<char> {
        if self.i >= self.str.len() {
            None
        } else {
            Some(self.as_str().chars().next().unwrap())
        }
    }

    pub(crate) fn offset(&self) -> usize {
        self.i
    }
}
