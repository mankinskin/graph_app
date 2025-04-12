use std::ops::Deref;

use crate::{
    graph::vertex::location::child::ChildLocation,
    path::accessors::{
        child::RootChildPos,
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
}
impl<R: PathRole> RootChildPos<R> for SubPath {
    fn root_child_pos(&self) -> usize {
        self.root_entry
    }
}
