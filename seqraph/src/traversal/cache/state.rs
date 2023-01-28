use crate::*;
use std::hash::Hash;

/// nodes generated during traversal.
//#[derive(Clone, Debug, PartialEq, Eq)]
//pub enum TraversalState<
//    R: ResultKind,
//> {
//    Start(StartState<R>),
//    Inner(InnerState<R>),
//}

//impl<
//    R: ResultKind,
//> TraversalState<R> {
//    pub fn entry_location(&self) -> Option<ChildLocation> {
//        match self {
//            Self::Inner(state) => state.entry_location(),
//            _ => None
//        }
//    }
//    pub fn prev_key(&self) -> Option<CacheKey> {
//        match self {
//            Self::Inner(state) => Some(state.prev),
//            _ => None
//        }
//    }
//    pub fn node_direction(&self) -> NodeDirection {
//        match self {
//            Self::Start(_)
//                => NodeDirection::BottomUp,
//            Self::Inner(state)
//                => state.node_direction(),
//        }
//    }
//}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InnerKind<
    R: ResultKind,
> {
    Parent(ParentState<R>),
    Child(ChildState<R>),
    End(EndState<R>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalState<
    R: ResultKind,
> {
    pub prev: CacheKey,
    pub matched: bool,
    pub kind: InnerKind<R>
}
impl<
    R: ResultKind,
> TraversalState<R> {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            InnerKind::Parent(state) => Some(state.path.root_child_location()),
            InnerKind::Child(state) => state.paths.get_path().role_leaf_child_location::<End>(),
            InnerKind::End(state) => state.entry_location(),
        }
    }
    pub fn prev_key(&self) -> CacheKey {
        self.prev
    }
    pub fn node_direction(&self) -> NodeDirection {
        match &self.kind {
            InnerKind::Parent(_)
                => NodeDirection::BottomUp,
            InnerKind::Child(_)
                => NodeDirection::TopDown,
            InnerKind::End(state) => state.node_direction()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StartState<
    R: ResultKind,
> {
    pub index: Child,
    pub query: R::Query,
    pub _ty: std::marker::PhantomData<R>,
}
impl<
    R: ResultKind,
> StartState<R> {
    pub fn new(index: Child, query: R::Query) -> Self {
        Self {
            index,
            query,
            _ty: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParentState<
    R: ResultKind,
> {
    pub path: R::Primer,
    pub query: R::Query,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ChildState<
    R: ResultKind,
> {
    pub root: CacheKey,
    pub paths: PathPair<R::Advanced, R::Query>,
}

// End types:
// - top down match-mismatch
// - top down match-query end
// - bottom up-no matching parents
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RangeKind {
    /// when the query has ended.
    QueryEnd,
    /// at a mismatch.
    Mismatch,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndKind<R: ResultKind> {
    /// when the query has ended.
    Range(RangeEnd<R>),
    /// at a mismatch.
    Postfix(PostfixEnd<R>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeEnd<R: ResultKind> {
    pub entry: ChildLocation,
    pub kind: RangeKind,
    pub path: R::Advanced,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostfixEnd<R: ResultKind> {
    pub path: R::Postfix,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndState<
    R: ResultKind,
> {
    pub root: CacheKey,
    pub kind: EndKind<R>,
    pub query: R::Query,
}

impl<
    R: ResultKind,
> EndState<R> {
    pub fn entry_location(&self) -> Option<ChildLocation> {
        match &self.kind {
            EndKind::Range(state) => Some(state.entry),
            EndKind::Postfix(_) => None,
        }
    }
    pub fn node_direction(&self) -> NodeDirection {
        match self.kind {
            EndKind::Range(_) => NodeDirection::TopDown,
            EndKind::Postfix(_) => NodeDirection::BottomUp,
        }
    }
    pub fn waiting_root_key(&self) -> Option<CacheKey> {
        match self.kind {
            EndKind::Range(_) => Some(self.root),
            EndKind::Postfix(_) => None,
        }
    }
}