use crate::{
    vertex::*,
    *,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartPath {
    First(ChildLocation, Child, usize),
    Path(ChildLocation, ChildPath, usize),
}
impl StartPath {
    pub fn entry(&self) -> ChildLocation {
        match self {
            Self::Path(entry, _, _) |
            Self::First(entry, _, _)
                => *entry,
        }
    }
    pub fn path(&self) -> ChildPath {
        match self {
            Self::Path(_, path, _) => path.clone(),
            _ => vec![],
        }
    }
    pub fn width(&self) -> usize {
        match self {
            Self::Path(_, _, width) |
            Self::First(_, _, width) => *width,
        }
    }
    pub fn width_mut(&mut self) -> &mut usize {
        match self {
            Self::Path(_, _, width) |
            Self::First(_, _, width) => width,
        }
    }
    pub(crate) fn prev_pos<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&self, trav: Trav) -> Option<usize> {
        let location = self.entry();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::index_prev(&pattern, location.sub_index)
    }
    pub(crate) fn is_complete<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&self, trav: Trav) -> bool {
        // todo: file bug, && behind match not recognized as AND
        // todo: respect match direction (need graph access
        let e = match self {
            Self::Path(_, path, _) => path.is_empty(),
            _ => true,
        };
        e && self.prev_pos::<_, _, D>(trav).is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRangePath {
    pub(crate) query: Pattern,
    pub(crate) entry: usize,
    pub(crate) start: ChildPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl QueryRangePath {
    pub fn complete(query: impl IntoPattern<Item = impl AsChild>) -> Self {
        let query = query.into_pattern();
        Self {
            entry: 0,
            exit: query.len() - 1,
            query,
            start: vec![],
            end: vec![],
        }
    }
    pub fn new_directed<D: MatchDirection, C: AsChild, P: IntoPattern<Item = C>>(query: P) -> Result<Self, NoMatch> {
        let entry = D::head_index(query.as_pattern_view());
        let query = query.into_pattern();
        (query.len() > 1).then(||
            Self {
                query,
                entry,
                start: vec![],
                exit: entry,
                end: vec![],
            }
        ).ok_or(NoMatch::SingleIndex)
    }
    pub fn get_entry(&self) -> Child {
        // todo: use path
        self.query.get(self.entry).cloned().expect("Invalid entry")
    }
    pub fn get_exit(&self) -> Child {
        // todo: use path
        self.query.get(self.exit).cloned().expect("Invalid exit")
    }
    pub(crate) fn get_end<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
        if let Some(next) = self.end.last() {
            trav.graph().expect_child_at(next)
        } else {
            self.query.get(self.exit).cloned().expect("Invalid exit")
        }
    }
    pub(crate) fn advance_next<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::index_next(&pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = D::index_next(&self.query, self.exit) {
            self.exit = next;
            true
        } else {
            false
        }
    }
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
}
#[derive(Debug, Clone, Eq)]
pub struct GraphRangePath {
    pub(crate) start: StartPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl GraphRangePath {
    pub fn get_exit_location(&self) -> ChildLocation {
        self.start.entry()
            .into_pattern_location()
            .to_child_location(self.exit)
    }
    pub fn get_entry_location(&self) -> ChildLocation {
        self.start.entry()
    }
    pub fn new(start: StartPath) -> Self {
        Self {
            exit: start.entry().sub_index,
            start,
            end: vec![],
        }
    }
    pub(crate) fn is_complete<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&self, trav: Trav) -> bool {
        self.start.is_complete::<_, _, D>(&trav) &&
            self.end.is_empty() &&
            self.next_pos::<_, _, D>(trav).is_none()
    }
    pub(crate) fn next_pos<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&self, trav: Trav) -> Option<usize> {
        let location = self.get_exit_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::index_next(&pattern, self.exit)
    }
    pub fn get_end_location(&self) -> ChildLocation {
        if self.end.is_empty() {
            self.get_exit_location()
        } else {
            self.end.last().unwrap().clone()
        }
    }
    pub(crate) fn get_end<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
    pub(crate) fn on_match<T: Tokenize>(&mut self, trav: impl Traversable<T>) {
        *self.start.width_mut() += self.get_end(trav).width;
    }
    pub(crate) fn advance_next<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            if let Some(next) = D::index_next(&pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = self.next_pos::<_, _, D>(&trav) {
            self.exit = next;
            true
        } else {
            false
        }
    }
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
    pub(crate) fn reduce_end<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(mut self, trav: Trav) -> FoundPath {
        let graph = trav.graph();
        //self.reduce_end_path::<T, D>(&*graph);
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::index_next(pattern.as_pattern_view(), location.sub_index).is_some() {
                self.end.push(location);
                break;
            }
        }
        FoundPath::new::<_, _, D>(&trav, self)
    }
}
#[derive(Clone)]
pub enum PathPair {
    GraphMajor(GraphRangePath, QueryRangePath),
    QueryMajor(QueryRangePath, GraphRangePath),
}
impl PathPair {
    pub fn from_mode(path: GraphRangePath, query: QueryRangePath, mode: bool) -> Self {
        if mode {
            Self::GraphMajor(path, query)
        } else {
            Self::QueryMajor(query, path)
        }
    }
    pub fn mode(&self) -> bool {
        matches!(self, Self::GraphMajor(_, _))
    }
    pub fn push_major(&mut self, location: ChildLocation) {
        match self {
            Self::GraphMajor(path, _) =>
                path.push_next(location),
            Self::QueryMajor(query, _) =>
                query.push_next(location),
        }
    }
    pub fn unpack(self) -> (GraphRangePath, QueryRangePath) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
}