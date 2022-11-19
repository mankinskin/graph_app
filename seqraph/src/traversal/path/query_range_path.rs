use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryRangePath {
    pub(crate) query: Pattern,
    pub(crate) entry: usize,
    pub(crate) start: ChildPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
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
pub(crate) trait QueryPath: TraversalQuery {
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
impl EntryPos for QueryRangePath {
    fn get_entry_pos(&self) -> usize {
        self.entry
    }
}
impl PatternEntry for QueryRangePath {
    fn get_entry_pattern(&self) -> &[Child] {
        self.query.borrow()
    }
}
impl HasStartPath for QueryRangePath {
    fn start_path(&self) -> &[ChildLocation] {
        self.start.borrow()
    }
}
impl PatternStart for QueryRangePath {}
impl ExitPos for QueryRangePath {
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
}
impl PatternExit for QueryRangePath {
    fn get_exit_pattern(&self) -> &[Child] {
        self.query.borrow()
    }
}
impl HasEndPath for QueryRangePath {
    fn end_path(&self) -> &[ChildLocation] {
        &self.end
    }
}
impl PatternEnd for QueryRangePath {}
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