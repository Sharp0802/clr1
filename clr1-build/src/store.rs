use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub struct Store<'a, T: Eq + Hash> {
    map: HashMap<&'a T, usize>,
    list: Vec<T>
}

impl<'a, T: Eq + Hash> Store<'a, T> {
    pub fn new() -> Self {
        Self { map: HashMap::new(), list: Vec::new() }
    }

    pub fn add(&mut self, v: T) -> usize {
        if let Some(&i) = self.map.get(&v) {
            i
        } else {
            let i = self.list.len();
            self.list.push(v);
            i
        }
    }
}

impl<'a, T: Eq + Hash> Index<usize> for Store<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
