use crate::*;
use super::*;

/// very similar to FoundPath
#[derive(Clone, Debug)]
pub(crate) enum MatchEnd {
    Path(StartPath),
    Full(Child),
}
impl MatchEnd {
    pub fn root(&self) -> Child {
        match self {
            MatchEnd::Path(start) => start.entry().parent,
            MatchEnd::Full(c) => *c,
        }
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
            MatchEnd::Full(child) => StartLeaf {
                entry: parent_entry,
                width: child.width(),
                child,
            }.into(),
        }
    }
}