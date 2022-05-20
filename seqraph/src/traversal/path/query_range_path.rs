use std::borrow::Borrow;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRangePath {
    pub(crate) query: Pattern,
    pub(crate) entry: usize,
    pub(crate) start: ChildPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
    pub(crate) finished: bool,
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
            finished: true
        }
    }
    pub fn new_directed<
        D: MatchDirection,
        P: IntoPattern,
    >(query: P) -> Result<Self, NoMatch> {
        let entry = D::head_index(query.borrow());
        let query = query.into_pattern();
        match query.len() {
            0 => Err(NoMatch::EmptyPatterns),
            1 => Err(NoMatch::SingleIndex),
            _ => Ok(Self {
                    query,
                    entry,
                    start: vec![],
                    exit: entry,
                    end: vec![],
                    finished: false
                })
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
            finished: true,
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
        &self.end
    }
}
impl PatternEnd for QueryRangePath {}
impl PathFinished for QueryRangePath {
    fn is_finished(&self) -> bool {
        self.finished
    }
    fn set_finished(&mut self) {
        self.finished = true;
    }
}
impl EndPathMut for QueryRangePath {
    fn end_path_mut(&mut self) -> &mut ChildPath {
        &mut self.end
    }
}
impl ExitMut for QueryRangePath {
    fn exit_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl End for QueryRangePath {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        self.get_pattern_end(trav)
    }
}
impl ReduciblePath for QueryRangePath {
    fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        if self.end.is_empty() {
            D::pattern_index_prev(self.query.borrow(), self.exit)
        } else {
            let location = *self.end.last().unwrap();
            let pattern = trav.graph().expect_pattern_at(&location);
            D::pattern_index_prev(pattern, location.sub_index)
        }
    }
}
impl AdvanceablePath for QueryRangePath {}