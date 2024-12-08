use crate::{
    direction::{
        Right,
        r#match::MatchDirection,
    },
    search::NoMatch,
    traversal::path::{
        accessors::role::End,
        BaseQuery,
        mutators::{
            append::PathAppend,
            move_path::root::MoveRootPos,
            pop::PathPop,
        },
        structs::rooted_path::{
            RootedPath,
            RootedRangePath,
            RootedRolePath,
            SearchPath,
            SubPath,
        },
    },
};
use crate::graph::vertex::pattern::{
    IntoPattern,
    Pattern,
};

use super::role_path::RolePath;

//#[derive(Debug, Clone, PartialEq, Eq, Hash)]
//pub struct QueryRangePath {
//    pub root: Pattern,
//    pub start: RolePath<Start>,
//    pub end: RolePath<End>,
//    pub pos: TokenLocation,
//}
pub type QueryRangePath = RootedRangePath<Pattern>;
pub type PatternPrefixPath = RootedRolePath<End, Pattern>;

//impl QueryRangePath {
//    pub fn new_postfix(query: impl IntoPattern, entry: usize) -> Self {
//        let query = query.into_pattern();
//        let len = query.len();
//        Self::new_range(query, entry, len)
//    }
//}

pub trait QueryPath:
BaseQuery
//+ LeafChildPosMut<End>
+ PathAppend
+ PathPop
+ MoveRootPos<Right, End>
{
    fn complete(pattern: impl IntoPattern) -> Self;
    fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(query: P) -> Result<Self, (NoMatch, Self)>;
}

impl QueryPath for QueryRangePath {
    fn complete(query: impl IntoPattern) -> Self {
        let query = query.into_pattern();
        let len = query.len();
        Self::new_range(query, 0, len - 1)
    }
    fn new_directed<D: MatchDirection, P: IntoPattern>(query: P) -> Result<Self, (NoMatch, Self)> {
        let entry = D::head_index(&query.borrow());
        let query = query.into_pattern();
        let len = query.len();
        let query = Self::new_range(query, entry, entry);
        match len {
            0 => Err((NoMatch::EmptyPatterns, query)),
            1 => Err((NoMatch::SingleIndex(*query.root.first().unwrap()), query)),
            _ => Ok(query),
        }
    }
}
impl QueryPath for PatternPrefixPath {
    fn complete(query: impl IntoPattern) -> Self {
        let pattern = query.into_pattern();
        Self {
            role_path: RolePath::from(
                SubPath::new(pattern.len() - 1),
            ),
            root: pattern,
        }
    }
    fn new_directed<D: MatchDirection, P: IntoPattern>(query: P) -> Result<Self, (NoMatch, Self)> {
        let pattern = query.into_pattern();
        let len = pattern.len();
        let p = Self {
            role_path: RolePath::from(
                SubPath::new(0),
            ),
            root: pattern,
        };
        match len {
            0 => Err((NoMatch::EmptyPatterns, p)),
            1 => Err((NoMatch::SingleIndex(*p.root.first().unwrap()), p)),
            _ => Ok(p),
        }
    }
}

pub trait RangePath: RootedPath {
    fn new_range(
        root: Self::Root,
        entry: usize,
        exit: usize,
    ) -> Self;
}

impl RangePath for QueryRangePath {
    fn new_range(
        root: Self::Root,
        entry: usize,
        exit: usize,
    ) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}

impl RangePath for SearchPath {
    fn new_range(
        root: Self::Root,
        entry: usize,
        exit: usize,
    ) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}
//impl PatternStart for QueryRangePath {}
//impl PatternEnd for QueryRangePath {}
//impl TraversalPath for QueryRangePath {
//    fn prev_exit_pos<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&self, trav: Trav) -> Option<usize> {
//        if self.end.is_empty() {
//            D::pattern_index_prev(self.query.borrow(), self.exit)
//        } else {
//            let location = *self.end.last().unwrap();
//            let pattern = trav.graph().expect_pattern_at(&location);
//            D::pattern_index_prev(pattern, location.sub_index)
//        }
//    }
//}
