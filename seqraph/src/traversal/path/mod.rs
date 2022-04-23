pub(crate) mod range_path;
pub(crate) use range_path::*;

use crate::{
    vertex::*,
    *,
};
pub trait RelativeDirection {
    type Direction: MatchDirection;
}
#[derive(Default)]
pub(crate) struct Front<D: MatchDirection>(std::marker::PhantomData<D>);
impl<D: MatchDirection> RelativeDirection for Front<D> {
    type Direction = D;
}
#[derive(Default)]
pub(crate) struct Back<D: MatchDirection>(std::marker::PhantomData<D>);
impl<D: MatchDirection> RelativeDirection for Back<D> {
    type Direction = <D as MatchDirection>::Opposite;
}

pub(crate) trait BorderPath: Wide {
    fn entry(&self) -> ChildLocation;
    fn path(&self) -> &[ChildLocation];
    /// true if path points to direct border in entry (path is empty)
    fn is_perfect(&self) -> bool {
        self.path().is_empty()
    }
    fn pattern<'a: 'g, 'g, 'x, T: Tokenize + 'g, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> Pattern {
        let graph = trav.graph();
        graph.expect_pattern_at(&self.entry())
    }
}
pub(crate) trait DirectedBorderPath<D: MatchDirection>: BorderPath {
    type BorderDirection: RelativeDirection;
    fn pattern_entry_outer_pos<P: IntoPattern>(pattern: P, entry: usize) -> Option<usize> {
        <Self::BorderDirection as RelativeDirection>::Direction::pattern_index_next(pattern, entry)
    }
    //fn pattern_entry_outer_context<P: IntoPattern>(pattern: P, entry: usize) -> ContextHalf {
    //    ContextHalf::try_new(<Self::BorderDirection as RelativeDirection>::Direction::front_context(pattern.borrow(), entry))
    //        .expect("GraphPath references border of index!")
    //}
    //fn pattern_outer_context<P: IntoPattern>(&self, pattern: P) -> ContextHalf {
    //    Self::pattern_entry_outer_context(pattern, self.entry().sub_index)
    //}
    fn pattern_outer_pos<P: IntoPattern>(&self, pattern: P) -> Option<usize> {
        Self::pattern_entry_outer_pos(pattern, self.entry().sub_index)
    }
    //fn outer_context<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> ContextHalf {
    //    self.pattern_outer_context(self.pattern(trav))
    //}
    fn outer_pos<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> Option<usize> {
        self.pattern_outer_pos(self.pattern(trav))
    }
    fn is_at_pattern_border<P: IntoPattern>(&self, pattern: P) -> bool {
        self.pattern_outer_pos(pattern).is_none()
    }
    fn pattern_is_complete<P: IntoPattern>(&self, pattern: P) -> bool {
        self.is_perfect() && self.is_at_pattern_border(pattern)
    }
    fn is_at_border<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> bool {
        self.outer_pos(trav).is_none()
    }
    fn is_complete<'a: 'g, 'g, T: Tokenize + 'a, Trav: Traversable<'a, 'g, T>>(&self, trav: &'a Trav) -> bool {
        self.is_perfect() && self.is_at_border(trav)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EndPath {
    pub(crate) entry: ChildLocation,
    pub(crate) path: ChildPath,
    pub(crate) width: usize,
}
//impl EndPath {
//    pub fn path_mut(&mut self) -> &mut ChildPath {
//        &mut self.path
//    }
//    pub fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}
impl BorderPath for EndPath {
    fn entry(&self) -> ChildLocation {
        self.entry
    }
    fn path(&self) -> &[ChildLocation] {
        self.path.as_slice()
    }
}
impl<D: MatchDirection> DirectedBorderPath<D> for EndPath {
    type BorderDirection = Front<D>;
}
impl Wide for EndPath {
    fn width(&self) -> usize {
        self.width
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StartPath {
    First {
        entry: ChildLocation,
        child: Child,
        width: usize,
    },
    Path {
        entry: ChildLocation,
        path: ChildPath,
        width: usize
    },
}
impl StartPath {
    pub fn width_mut(&mut self) -> &mut usize {
        match self {
            Self::Path { width, .. } |
            Self::First { width , ..} => width,
        }
    }
}
impl BorderPath for StartPath {
    fn entry(&self) -> ChildLocation {
        match self {
            Self::Path { entry, .. } |
            Self::First { entry, .. }
                => *entry,
        }
    }
    fn path(&self) -> &[ChildLocation] {
        match self {
            Self::Path { path, .. } => path.as_slice(),
            _ => &[],
        }
    }
}
impl<D: MatchDirection> DirectedBorderPath<D> for StartPath {
    type BorderDirection = Back<D>;
}
impl Wide for StartPath {
    fn width(&self) -> usize {
        match self {
            Self::Path { width, .. } |
            Self::First { width, .. } => *width,
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub(crate) struct GraphRangePath {
    pub(crate) start: StartPath,
    pub(crate) inner_width: usize,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
    pub(crate) end_width: usize,
}
impl<'a: 'g, 'g> GraphRangePath {
    pub fn new(start: StartPath) -> Self {
        Self {
            exit: start.entry().sub_index,
            start,
            end: vec![],
            end_width: 0,
            inner_width: 0,
        }
    }
    pub fn into_paths(self) -> (StartPath, EndPath) {
        let entry = self.start.entry();
        let mut exit = entry.clone();
        exit.sub_index = self.exit;
        (
            self.start,
            EndPath {
                entry: exit,
                path: self.end,
                width: self.end_width,
            }
        )
    }
    pub(crate) fn is_complete<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> bool {
        let pattern = self.start.pattern(trav);
        DirectedBorderPath::<D>::pattern_is_complete(&self.start, &pattern[..]) &&
            self.end.is_empty() &&
            <EndPath as DirectedBorderPath<D>>::pattern_entry_outer_pos(pattern, self.exit).is_none()
    }
    pub(crate) fn next_pos<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_next(pattern, self.exit)
    }
    /// true if points to a match end position
    //pub(crate) fn has_end_match<
    //    T: Tokenize + 'a,
    //    Trav: Traversable<'a, 'g, T>,
    //    D: MatchDirection + 'a,
    //>(&self, trav: &'a Trav) -> bool {
    //    !self.end.is_empty() || self.prev_pos::<_, _, D>(trav) >= Some(self.start.entry().sub_index)
    //}
    fn get_exit_location(&self) -> ChildLocation {
        self.start.entry()
            .into_pattern_location()
            .to_child_location(self.exit)
    }
    fn get_end_location(&self) -> ChildLocation {
        if self.end.is_empty() {
            self.get_exit_location()
        } else {
            self.end.last().unwrap().clone()
        }
    }
    //pub(crate) fn on_match<
    //    T: Tokenize + 'a,
    //    D: MatchDirection + 'a,
    //    Trav: Traversable<'a, 'g, T>,
    //>(&mut self, trav: &'a Trav) {
    //    let width = self.get_end::<_, D, _>(trav).width;
    //    let wmut = if self.end.is_empty() {
    //        &mut self.inner_width
    //    } else {
    //        &mut self.end_width
    //    };
    //    *wmut += width;
    //}
}
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
    fn get_exit(&self) -> Child {
        self.query[self.exit]
    }
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
impl Wide for GraphRangePath {
    fn width(&self) -> usize {
        self.start.width() + self.end_width
    }
}
#[derive(Clone, Debug)]
pub(crate) enum PathPair<
    Q: QueryPath,
    G: GraphPath,
> {
    GraphMajor(G, Q),
    QueryMajor(Q, G),
}
impl<
    Q: QueryPath,
    G: GraphPath,
> PathPair<Q, G> {
    pub fn from_mode(path: G, query: Q, mode: bool) -> Self {
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
    pub fn unpack(self) -> (G, Q) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
    pub(crate) fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> QueryResult<Q> {
        match self {
            Self::GraphMajor(path, query) |
            Self::QueryMajor(query, path) => {
                QueryResult::new(
                    FoundPath::new::<_, D, _>(trav, path.reduce_mismatch::<_, D, _>(trav).into()),
                    query.reduce_mismatch::<_, D, _>(trav),
                )
            }
        }
    }
}