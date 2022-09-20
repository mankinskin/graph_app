use crate::*;
use super::*;

//pub(crate) trait NotStartPath {}
//impl NotStartPath for StartLeaf {}

pub(crate) trait MatchEndPath: NodePath + PathComplete + Into<StartPath> + From<StartPath> {}
impl<T: NodePath + PathComplete + Into<StartPath> + From<StartPath>> MatchEndPath for T {}

/// Used to represent results after traversal with only a start path
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub(crate) enum MatchEnd<P: MatchEndPath> {
    Path(P),
    Complete(Child),
}
pub(crate) trait IntoMatchEndStartPath {
    fn into_mesp(self) -> MatchEnd<StartPath>;
}
pub(crate) trait FromMatchEnd<P: MatchEndPath> {
    fn from_match_end(match_end: MatchEnd<P>, start: StartPath) -> Self;
}
impl<P: MatchEndPath> FromMatchEnd<P> for MatchEnd<P> {
    fn from_match_end(match_end: MatchEnd<P>, start: StartPath) -> Self {
        match_end
    }
}
impl<P: MatchEndPath> FromMatchEnd<P> for OriginPath<P> {
    fn from_match_end(match_end: MatchEnd<P>, origin: StartPath) -> Self {
        OriginPath {
            match_end,
            origin,
        }
    }
}
impl<P: MatchEndPath> IntoMatchEndStartPath for MatchEnd<P> {
    fn into_mesp(self) -> MatchEnd<StartPath> {
        match self {
            MatchEnd::Path(p) => MatchEnd::Path(p.into()),
            MatchEnd::Complete(c) => MatchEnd::Complete(c)
        }
    }
}
impl<P: MatchEndPath> IntoMatchEndStartPath for OriginPath<P> {
    fn into_mesp(self) -> MatchEnd<StartPath> {
        self.match_end.into_mesp()
    }
}
impl From<OriginPath<StartPath>> for MatchEnd<StartPath> {
    fn from(start: OriginPath<StartPath>) -> Self {
        start.match_end
    }
}
impl<P: MatchEndPath> From<P> for MatchEnd<P> {
    fn from(start: P) -> Self {
        MatchEnd::Path(start)
    }
}
impl<P: MatchEndPath> RootChild for MatchEnd<P> {
    fn root_child(&self) -> Child {
        match self {
            MatchEnd::Path(start) => start.root_child(),
            MatchEnd::Complete(c) => *c,
        }
    }
}
impl<P: MatchEndPath> MatchEnd<P> {
    //#[allow(unused)]
    //pub fn into_path(self) -> Option<StartPath> {
    //    match self {
    //        Self::Path(path) => Some(path),
    //        _ => None,
    //    }
    //}
    pub fn get_path(&self) -> Option<&P> {
        match self {
            Self::Path(start) => Some(start),
            _ => None,
        }
    }
    //pub fn into_result<R: ResultKind>(self, start: StartPath) -> R::Result<P> {
    //    match self {
    //        Self::Path(start) => Some(start),
    //        _ => None,
    //    }
    //}
}
impl<P: MatchEndPath> PathComplete for MatchEnd<P> {
    fn complete<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}
impl<P: MatchEndPath> PathReduce for MatchEnd<P> {
    fn reduce<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        if let Some(c) = self.get_path()
            .and_then(|p| p.complete::<_, D, _>(trav))
        {
            *self = Self::Complete(c);
        }
    }
}
impl<P: MatchEndPath> PathAppend for MatchEnd<P> {
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
            MatchEnd::Complete(child) => StartLeaf {
                entry: parent_entry,
                width: child.width(),
                child,
            }.into(),
        }
    }
}