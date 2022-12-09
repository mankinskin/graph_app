use crate::*;
use super::*;

//pub trait NotStartPath {}
//impl NotStartPath for StartLeaf {}

pub trait MatchEndPath: NodePath + PathComplete + PathAppend<Result=StartPath> + Into<StartPath> + From<StartPath> + From<StartLeaf> + Into<FoundPath> + Hash + Sync + Send {}
impl<T: NodePath + PathComplete + PathAppend<Result=StartPath> + Into<StartPath> + From<StartPath> + From<StartLeaf> + Into<FoundPath> + Hash + Sync + Send> MatchEndPath for T {}

/// Used to represent results after traversal with only a start path
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum MatchEnd<P: MatchEndPath> {
    Path(P),
    Complete(Child),
}
pub trait IntoMatchEndStartPath {
    fn into_mesp(self) -> MatchEnd<StartPath>;
}
impl<P: MatchEndPath> RangePath for MatchEnd<P> {
    fn into_complete(self) -> Option<Child> {
        match self {
            Self::Complete(index) => Some(index),
            _ => None,
        }
    }
}
//impl<P: RangePath> Into<FoundPath> for MatchEnd<P> {
//    fn into(self) -> FoundPath {
//        match self {
//            MatchEnd::Path(path) => p.into(),
//            MatchEnd::Complete(c) => FoundPath::Complete(c),
//        }
//    }
//}
//impl<P: RangePath> Into<FoundPath> for OriginPath<P> {
//    fn into(self) -> FoundPath {
//        self.postfix.into()
//    }
//}
//impl<P: MatchEndPath> FromSearchPath for MatchEnd<P> {
//    fn from_search_path<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(path: SearchPath, trav: Trav) -> Self {
//        Self {
//            origin: path.start_match_path().clone(),
//            postfix: P::from_search_path(path, trav),
//        }
//    }
//}
//impl<P: MatchEndPath> FromMatchEnd<P> for OriginPath<MatchEnd<P>> {
//    fn from_match_end(match_end: MatchEnd<P>, origin: StartPath) -> Self {
//        OriginPath {
//            postfix: match_end,
//            origin,
//        }
//    }
//}
impl<P: MatchEndPath> IntoMatchEndStartPath for MatchEnd<P> {
    fn into_mesp(self) -> MatchEnd<StartPath> {
        match self {
            MatchEnd::Path(p) => MatchEnd::Path(p.into()),
            MatchEnd::Complete(c) => MatchEnd::Complete(c)
        }
    }
}
impl<P: MatchEndPath> IntoRangePath for MatchEnd<P> {
    type Result = FoundPath;
    fn into_range_path(self) -> Self::Result {
        FoundPath::from(self.into_mesp())
    }
}
impl<P: MatchEndPath> IntoMatchEndStartPath for OriginPath<MatchEnd<P>> {
    fn into_mesp(self) -> MatchEnd<StartPath> {
        self.postfix.into_mesp()
    }
}
impl From<OriginPath<MatchEnd<StartPath>>> for MatchEnd<StartPath> {
    fn from(start: OriginPath<MatchEnd<StartPath>>) -> Self {
        start.postfix
    }
}
impl From<MatchEnd<StartLeaf>> for MatchEnd<StartPath> {
    fn from(start: MatchEnd<StartLeaf>) -> Self {
        match start {
            MatchEnd::Path(leaf) => MatchEnd::Path(leaf.into()),
            MatchEnd::Complete(c) => MatchEnd::Complete(c)
        }
    }
}
impl<P: MatchEndPath + From<Q>, Q: Into<StartPath>> From<Q> for MatchEnd<P> {
    fn from(start: Q) -> Self {
        MatchEnd::Path(P::from(start))
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
    #[allow(unused)]
    pub fn unwrap_path(self) -> P {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }.unwrap()
    }
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
        Trav: Traversable<T>,
    >(&self, _trav: &'a Trav) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}

impl<P: MatchEndPath + PathPop<Result=Self>> PathReduce for MatchEnd<P> {
    fn into_reduced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(self, trav: &'a Trav) -> Self {
        if let Some(c) = match self.get_path() {
            Some(p) => p.complete::<_, D, _>(trav),
            None => None,
        } {
            MatchEnd::Complete(c)
        } else {
            self    
        }
        //if let MatchEnd::Path(path) = self {
        //    path.pop_path::<_, D, _>(trav)
        //} else {
        //    self    
        //}
    }
}

impl<P: MatchEndPath + PathAppend> PathAppend for MatchEnd<P> {
    type Result = <P as PathAppend>::Result;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        match self {
            MatchEnd::Path(path) => path.append::<_, D, _>(trav, parent_entry),
            MatchEnd::Complete(child) => StartLeaf {
                entry: parent_entry,
                width: child.width(),
                child,
                token_pos: 0,
            }.into(),
        }
    }
}

impl<P: MatchEndPath + PathPop<Result=Self>> PathPop for MatchEnd<P> {
    type Result = Result<Self, Child>;
    fn pop_path<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav) -> Self::Result {
        match self {
            MatchEnd::Path(path) => Ok(path.pop_path::<_, D, _>(trav)),
            MatchEnd::Complete(child) => Err(child),
        }
    }
}
impl<P: MatchEndPath + GetCacheKey> GetCacheKey for MatchEnd<P> {
    fn cache_key(&self) -> CacheKey {
        match self {
            MatchEnd::Path(path) => path.cache_key(),
            MatchEnd::Complete(c) => c.cache_key(),
        }
    }
}