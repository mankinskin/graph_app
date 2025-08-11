use std::ops::Deref;

use crate::{
    graph::vertex::location::child::ChildLocation,
    path::accessors::{
        child::RootChildIndex,
        role::PathRole,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubPath {
    pub root_entry: usize,
    pub path: Vec<ChildLocation>,
}

impl Deref for SubPath {
    type Target = Vec<ChildLocation>;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl SubPath {
    pub fn new(root_entry: usize) -> Self {
        Self {
            root_entry,
            path: vec![],
        }
    }
    pub fn pop_while(
        &mut self,
        condition: impl Fn(&ChildLocation) -> bool,
    ) {
        while self.path.last().map(&condition).unwrap_or_default() {
            self.path.pop();
        }
    }
}
impl<R: PathRole> RootChildIndex<R> for SubPath {
    fn root_child_index(&self) -> usize {
        self.root_entry
    }
}
