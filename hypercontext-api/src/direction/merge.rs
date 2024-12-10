use crate::graph::vertex::{
    child::Child,
    pattern::Pattern,
};

pub trait Merge {
    fn split_front(self) -> Option<(Child, Pattern)>;
    fn split_back(self) -> Option<(Child, Pattern)>;
}

impl Merge for Child {
    fn split_front(self) -> Option<(Child, Pattern)> {
        Some((self, vec![]))
    }
    fn split_back(self) -> Option<(Child, Pattern)> {
        Some((self, vec![]))
    }
}

impl Merge for Pattern {
    fn split_front(self) -> Option<(Child, Pattern)> {
        let mut p = self.into_iter();
        let first = p.next();
        first.map(|last| (last, p.collect()))
    }
    fn split_back(mut self) -> Option<(Child, Pattern)> {
        let last = self.pop();
        last.map(|last| (last, self))
    }
}