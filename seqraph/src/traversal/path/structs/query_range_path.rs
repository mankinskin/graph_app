use crate::*;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryRangePath {
    pub root: Pattern,
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
}
impl QueryRangePath {
    pub fn new_postfix(query: impl IntoPattern, entry: usize) -> Self {
        let query = query.into_pattern();
        let len = query.len();
        Self::new_range(query, entry, len)
    }
}
pub trait QueryPath:
    BaseQuery
    //+ LeafChildPosMut<End>
    + PathAppend
    + PathPop
    + AdvanceRootPos<End>
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
        Self::new_range(query, 0, len-1)
    }
    fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(query: P) -> Result<Self, (NoMatch, Self)> {
        let entry = D::head_index(query.borrow());
        let query = query.into_pattern();
        let first = *query.first().unwrap();
        let len = query.len();
        let query = Self::new_range(query, entry, entry);
        match len {
            0 => Err((NoMatch::EmptyPatterns, query)),
            1 => Err((NoMatch::SingleIndex(first), query)),
            _ => Ok(query)
        }
    }
}
pub trait RangePath: RootedPath {
    fn new_range(root: Self::Root, entry: usize, exit: usize) -> Self;
}
impl RangePath for QueryRangePath {
    fn new_range(root: Self::Root, entry: usize, exit: usize) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}
impl RangePath for SearchPath {
    fn new_range(root: Self::Root, entry: usize, exit: usize) -> Self {
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