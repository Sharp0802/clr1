use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub struct Store<T: Eq + Hash> {
    map: HashMap<T, usize>,
    list: Vec<T>
}

impl<T: Eq + Hash> Store<T> {
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

impl<T: Eq + Hash> Index<usize> for Store<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
