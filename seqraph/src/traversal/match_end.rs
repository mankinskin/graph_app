use crate::*;
use super::*;

/// Used to represent results after traversal with only a start path
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub(crate) enum MatchEnd {
    Path(StartPath),
    Complete(Child),
}
impl MatchEnd {
    pub fn root(&self) -> Child {
        match self {
            MatchEnd::Path(start) => start.entry().parent,
            MatchEnd::Complete(c) => *c,
        }
    }
    //pub fn root<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    Trav: Traversable<'a, 'g, T>
    //>(self, trav: &Trav) -> FoundPath {
    //    match self {
    //        MatchEnd::Path(start) => FoundPath::from(start),
    //        MatchEnd::Complete(c) => *c,
    //    }
    //}
    pub fn into_path(self) -> Option<StartPath> {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }
    }
    pub fn get_path(&self) -> Option<&StartPath> {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }
    }
    pub fn reduce_start<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Self {
        self.get_path()
            .and_then(|p| p.reduce::<_, D, _>(trav))
            .map(Self::Complete)
            .unwrap_or_else(|| self)
        
    }
}
impl From<StartPath> for MatchEnd {
    fn from(start: StartPath) -> Self {
        MatchEnd::Path(start)
    }
}
impl PathAppend for MatchEnd {
    type Result = StartPath;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        match self {
            MatchEnd::Path(path) => path.append::<_, D, _>(trav, parent_entry),
                //path.get_leaf()
                //    .and_then(|leaf| {
                //        let graph = trav.graph();
                //        let pattern = graph.expect_pattern_at(leaf.entry);
                //        (leaf.entry.sub_index == D::head_index(pattern))
                //            .then(|| MatchEnd::Complete(leaf.entry.parent))
                //    })
                //    .unwrap_or_else(|| MatchEnd::Path(path))
            MatchEnd::Complete(child) => StartLeaf {
                entry: parent_entry,
                width: child.width(),
                child,
            }.into(),
        }
    }
}