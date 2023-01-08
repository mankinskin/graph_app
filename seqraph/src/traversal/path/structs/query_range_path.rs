use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryRangePath {
    pub query: Pattern,
    pub entry: usize,
    pub start: LocationPath,
    pub exit: usize,
    pub end: LocationPath,
}
impl<
    'a: 'g,
    'g,
> QueryRangePath {
    pub fn postfix(query: impl IntoPattern, entry: usize) -> Self {
        let query = query.into_pattern();
        Self {
            entry,
            exit: query.len() - 1,
            query,
            start: vec![],
            end: vec![],
        }
    }
    pub fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(query: P) -> Result<Self, (NoMatch, Self)> {
        let entry = D::head_index(query.borrow());
        let query = query.into_pattern();
        let mk_path = |query| Self {
                query,
                entry,
                start: vec![],
                exit: entry,
                end: vec![],
            };
        match query.len() {
            0 => Err((NoMatch::EmptyPatterns, mk_path(query))),
            1 => Err((NoMatch::SingleIndex(*query.first().unwrap()), mk_path(query))),
            _ => Ok(mk_path(query))
        }
    }
}
pub trait QueryPath: BaseQuery {
    fn complete(pattern: impl IntoPattern) -> Self;
}

impl QueryPath for QueryRangePath {
    fn complete(query: impl IntoPattern) -> Self {
        let query = query.into_pattern();
        Self {
            entry: 0,
            exit: query.len(),
            query,
            start: vec![],
            end: vec![],
        }
    }
}
//impl HasRootedPath for QueryRangePath {
//    fn child_path(&self) -> &[ChildLocation] {
//        self.start.borrow()
//    }
//}
//impl PatternStart for QueryRangePath {}
//impl HasRootedPath for QueryRangePath {
//    fn child_path(&self) -> &[ChildLocation] {
//        &self.end
//    }
//}
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
impl AdvanceWidth for QueryRangePath {
    fn advance_width(&mut self, _width: usize) {
    }
}