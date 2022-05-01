use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
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
        D: MatchDirection + 'a,
        P: IntoPattern,
    >(query: P) -> Result<Self, NoMatch> {
        let entry = D::head_index(query.borrow());
        let query = query.into_pattern();
        match query.len() {
            0 => Err(NoMatch::EmptyPatterns),
            1 => Err(NoMatch::SingleIndex),
            _ => 
            Ok(Self {
                query,
                entry,
                start: vec![],
                exit: entry,
                end: vec![],
            })
        }
    }
    #[allow(unused)]
    pub(crate) fn get_advance<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Option<(Child, Self)> {
        if self.advance_next::<_, D, _>(trav) {
            Some((self.get_end::<_, D, _>(trav), self))
        } else {
            None
        }
    }
}
pub trait QueryPath: TraversalQuery {
    fn complete(pattern: impl IntoPattern) -> Self;
}

impl QueryPath for QueryRangePath {
    fn complete(query: impl IntoPattern) -> Self {
        let query = query.into_pattern();
        Self {
            entry: 0,
            exit: query.len() - 1,
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
    fn get_start_path(&self) -> &[ChildLocation] {
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
    fn get_end_path(&self) -> &[ChildLocation] {
        self.end.borrow()
    }
}
impl PatternEnd for QueryRangePath {}
impl RangePath for QueryRangePath {
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end.push(location);
                break;
            }
        }
        if self.end.is_empty() {
            self.exit = self.prev_pos::<_, D, _>(trav).unwrap();
        }

        self
    }
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        if self.end.is_empty() {
            D::pattern_index_prev(self.query.borrow(), self.exit)
        } else {
            let location = self.end.last().unwrap().clone();
            let pattern = trav.graph().expect_pattern_at(&location);
            D::pattern_index_prev(pattern, location.sub_index)
        }
    }
    fn advance_next<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = D::pattern_index_next(self.query.borrow(), self.exit) {
            self.exit = next;
            true
        } else {
            false
        }
    }
}