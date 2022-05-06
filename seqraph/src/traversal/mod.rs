pub(crate) mod bft;
pub(crate) mod dft;
pub(crate) mod path;

pub(crate) use bft::*;
pub(crate) use dft::*;
pub use path::*;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::{
    ControlFlow,
    Deref,
    DerefMut,
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use itertools::Itertools;

use crate::{
    Child,
    ChildLocation,
    Tokenize,
    Hypergraph,
    Vertexed,
    MatchDirection,
    TraversalOrder, QueryResult, FoundPath, HypergraphRef, Wide, PatternLocation, Pattern,
};

pub(crate) trait ToTraversalNode<
    Q: TraversalQuery,
    G: TraversalPath,
    >: Clone + Into<TraversalNode<Q, G>> {
    fn query_node(query: Q) -> Self;
    fn root_node(query: Q, start: Option<StartPath>, entry: ChildLocation) -> Self;
    fn match_node(path: G, query: Q, old_query: Q) -> Self;
    fn end_node(found: Option<QueryResult<Q>>) -> Self;
    fn mismatch_node(paths: PathPair<Q, G>) -> Self;
}

#[derive(Clone, Debug)]
pub(crate) enum TraversalNode<
    Q: TraversalQuery,
    G: TraversalPath,
> {
    Query(Q),
    Root(Q, Option<StartPath>, ChildLocation),
    Match(G, Q, Q),
    End(Option<QueryResult<Q>>),
    Mismatch(PathPair<Q, G>),
}
impl<
    Q: TraversalQuery,
    G: TraversalPath,
> ToTraversalNode<Q, G> for TraversalNode<Q, G> {
    fn query_node(query: Q) -> Self {
        Self::Query(query)
    }
    fn root_node(query: Q, start: Option<StartPath>, entry: ChildLocation) -> Self {
        Self::Root(query, start, entry)
    }
    fn match_node(path: G, query: Q, old_query: Q) -> Self {
        Self::Match(path, query, old_query)
    }
    fn end_node(found: Option<QueryResult<Q>>) -> Self {
        Self::End(found)
    }
    fn mismatch_node(paths: PathPair<Q, G>) -> Self {
        Self::Mismatch(paths)
    }
}

pub(crate) type MatchNode = TraversalNode<QueryRangePath, GraphRangePath>;
pub(crate) type IndexingNode<Q> = TraversalNode<Q, GraphRangePath>;

pub trait Traversable<'a: 'g, 'g, T: Tokenize>: Sized + 'a {
    type Guard: Traversable<'g, 'g, T> + Deref<Target=Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard;
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for &'a Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for &'a mut Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        &*self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for RwLockReadGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        &*self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        &**self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for HypergraphRef<T> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.read().unwrap()
    }
}

pub(crate) trait TraversableMut<'a: 'g, 'g, T: Tokenize> : Traversable<'a, 'g, T> {
    type GuardMut: TraversableMut<'g, 'g, T> + Deref<Target=Hypergraph<T>> + DerefMut;
    fn graph_mut(&'g mut self) -> Self::GuardMut;
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for Hypergraph<T> {
    type Guard = &'g Self;
    fn graph(&'g self) -> Self::Guard {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for Hypergraph<T> {
    type GuardMut = &'g mut Self;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for &'a mut Hypergraph<T> {
    type GuardMut = &'g mut Hypergraph<T>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        *self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type GuardMut = &'g mut Hypergraph<T>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        &mut **self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for HypergraphRef<T> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.write().unwrap()
    }
}
pub(crate) type Folder<'a, 'g, T, D, Q, Ty>
    = <Ty as DirectedTraversalPolicy<'a, 'g, T, D, Q>>::Folder;
pub(crate) type FolderNode<'a, 'g, T, D, Q, Ty>
    = <Folder<'a, 'g, T, D, Q, Ty> as TraversalFolder<'a, 'g, T, D, Q>>::Node;
pub(crate) trait FolderQ<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery,
> {
    type Query: TraversalQuery;
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    Ty: TraversalFolder<'a, 'g, T, D, Q>,
> FolderQ<'a, 'g, T, D, Q> for Ty {
    type Query = Q;
}
pub(crate) type FolderQuery<'a, 'g, T, D, Q, Ty> =
    <Folder<'a, 'g, T, D, Q, Ty> as FolderQ<'a, 'g, T, D, Q>>::Query;
pub(crate) type FolderPath<'a, 'g, T, D, Q, Ty>
    = <Folder<'a, 'g, T, D, Q, Ty> as TraversalFolder<'a, 'g, T, D, Q>>::Path;
pub(crate) trait TraversalIterator<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Q, Trav=Trav>,
>: Iterator<Item = (usize, FolderNode<'a, 'g, T, D, Q, S>)>
{
    fn new(trav: &'a Trav, root: FolderNode<'a, 'g, T, D, Q, S>) -> Self;
    fn iter_children(trav: &'a Trav, node: &FolderNode<'a, 'g, T, D, Q, S>) -> Vec<FolderNode<'a, 'g, T, D, Q, S>> {
        match node.clone().into() {
            TraversalNode::Query(query) =>
                S::query_start(
                    trav,
                    query,
                ),
            TraversalNode::Root(query, start, parent_entry) =>
                S::root_successor_nodes(
                    trav,
                    query,
                    start,
                    parent_entry,
                ),
            TraversalNode::Match(path, query, _prev_query) =>
                S::after_match(
                    trav,
                    PathPair::GraphMajor(path, query),
                ),
            _ => vec![],
        }
    }
}
pub(crate) trait BandExpandingPolicy<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
> {
    fn expand_band(location: PatternLocation, pattern: &Pattern) -> (ChildLocation, Child);
    fn map_batch(batch: impl IntoIterator<Item=(ChildLocation, Child)>) -> Vec<(ChildLocation, Child)> {
        batch.into_iter().collect_vec()
    }
}
pub(crate) struct PostfixExpandingPolicy<D: MatchDirection> {
    _ty: std::marker::PhantomData<D>,
}

pub(crate) trait BandIterator<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
>: Iterator<Item = (Option<ChildLocation>, ChildLocation, Child)>
{
    fn new(trav: &'a Trav, root: Child) -> Self;
    fn next_children(trav: &'a Trav, index: Child) -> Vec<(ChildLocation, Child)> {
        P::map_batch(
            trav.graph()
                .expect_children_of(index)
                .into_iter()
                .map(|(pid, pattern)|
                    P::expand_band(PatternLocation::new(index, *pid), pattern)
                )
        )
    }
}
impl <
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection,
> BandExpandingPolicy<'a, 'g, T, Trav> for PostfixExpandingPolicy<D> {
    fn expand_band(location: PatternLocation, pattern: &Pattern) -> (ChildLocation, Child) {
        let last = D::last_index(pattern);
        (location.to_child_location(last), pattern[last].clone())
    }
    fn map_batch(batch: impl IntoIterator<Item=(ChildLocation, Child)>) -> Vec<(ChildLocation, Child)> {
        batch.into_iter()
            .sorted_by(|a, b|
                a.1.width().cmp(&b.1.width())
            )
            .collect_vec()
    }
}
pub(crate) struct BandExpandingIterator<'a: 'g, 'g, T, Trav, P>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
{
    trav: &'a Trav,
    queue: VecDeque<(ChildLocation, Child)>,
    last: (Option<ChildLocation>, Child),
    _ty: std::marker::PhantomData<(&'g T, P)>
}
pub(crate) type PostfixIterator<'a, 'g, T, D, Trav>
    = BandExpandingIterator<'a, 'g, T, Trav, PostfixExpandingPolicy<D>>;

impl<'a: 'g, 'g, T, Trav, P> BandIterator<'a, 'g, T, Trav, P> for BandExpandingIterator<'a, 'g, T, Trav, P>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
{
    fn new(trav: &'a Trav, root: Child) -> Self {
        Self {
            trav,
            queue: VecDeque::new(),
            last: (None, root),
            _ty: Default::default(),
        }
    }
}
impl<'a: 'g, 'g, T, Trav, P> Iterator for BandExpandingIterator<'a, 'g, T, Trav, P>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    P: BandExpandingPolicy<'a, 'g, T, Trav>,
{
    type Item = (Option<ChildLocation>, ChildLocation, Child);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (last_location, last_node) = &mut self.last;
        let mut segment = None;
        if self.queue.is_empty() {
            segment = last_location.take();
            self.queue.extend(
                <Self as BandIterator<T, Trav, P>>::next_children(&self.trav, last_node.clone())
            )
        }
        if let Some((location, node)) = self.queue.pop_front() {
            *last_location = Some(location);
            *last_node = node.clone();
            Some((segment, location, node))
        } else {
            None
        }
    }
}
pub(crate) trait DirectedTraversalPolicy<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a, Q: TraversalQuery + 'a>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Folder: TraversalFolder<'a, 'g, T, D, Q, Trav=Self::Trav>;
    fn end_op(
        trav: &'a Self::Trav,
        query: FolderQuery<'a, 'g, T, D, Q, Self>,
        start_path: StartPath,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>>;
    fn parent_nodes(
        trav: &'a Self::Trav,
        query: FolderQuery<'a, 'g, T, D, Q, Self>,
        start: Option<StartPath>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {

        let graph = trav.graph();
        let start_index = match start {
            Some(StartPath::First { entry, .. }) |
            Some(StartPath::Path { entry, .. }) =>
                entry.parent,
            None => query.get_entry()
        };
        let vertex = start_index.vertex(&graph).clone();
        let mut parents = vertex.get_parents()
            .into_iter()
            .map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent.pattern_indices
                    .iter()
                    .cloned()
                    .map(move |pi| {
                        ChildLocation::new(p, pi.pattern_id, pi.sub_index)
                    })
            })
            .flatten()
            .collect_vec();
        // try parents in ascending width (might not be needed in indexing)
        parents.sort_unstable_by(|a, b| TraversalOrder::cmp(a, b));
        parents.into_iter()
            .map(|p|
                ToTraversalNode::root_node(
                    query.clone(),
                    start.clone(),
                    p,
                )
            )
            .collect_vec()
    }
    fn root_successor_nodes(
        trav: &'a Self::Trav,
        old_query: FolderQuery<'a, 'g, T, D, Q, Self>,
        old_start: Option<StartPath>,
        parent_entry: ChildLocation,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        let start_index = old_query.get_entry();
        let graph = trav.graph();
        let pre_start = match old_start.clone() {
            Some(StartPath::First { entry, width, .. }) => {
                let pattern = graph.expect_pattern_at(entry);
                //println!("first {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                StartPath::Path {
                    entry: parent_entry,
                    path: if entry.sub_index != D::head_index(&pattern) {
                        vec![entry]
                    } else {
                        vec![]
                    },
                    width,
                }
            },
            Some(StartPath::Path { entry, mut path, width }) => {
                //println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                let pattern = graph.expect_pattern_at(entry);
                if entry.sub_index != D::head_index(&pattern) || !path.is_empty() {
                    path.push(entry);
                }
                StartPath::Path {
                    entry: parent_entry,
                    path,
                    width,
                }
            },
            None => {
                //println!("start {} -> {}, {}", start_index.index, parent_entry.parent.index, start_index.width);
                StartPath::First {
                    entry: parent_entry,
                    child: start_index,
                    width: start_index.width,
                }
            }
        };
        drop(graph);
        let path = FolderPath::<'a, 'g, T, D, Q, Self>::from(pre_start);
        IntoSequenceIterator::<_, D, _>::into_seq_iter(path, trav).next()
            .map(|path|
                Self::match_end(&trav, PathPair::GraphMajor(path, old_query.clone()))
            )
            .unwrap_or_else(|path|
                Self::index_end(trav, old_query.clone(), path)
            )
    }
    fn query_start(
        trav: &'a Self::Trav,
        query: FolderQuery<'a, 'g, T, D, Q, Self>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        IntoSequenceIterator::<_, D, _>::into_seq_iter(query, trav).next()
            .map(|query|
                Self::parent_nodes(
                    trav,
                    query,
                    None,
                )
            )
            .unwrap_or_else(|_|
                vec![ToTraversalNode::end_node(None)]
            )
    }
    fn after_match(
        trav: &'a Self::Trav,
        paths: PathPair<FolderQuery<'a, 'g, T, D, Q, Self>, FolderPath<'a, 'g, T, D, Q, Self>>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        let mode = paths.mode();
        let (path, query) = paths.unpack();
        IntoSequenceIterator::<_, D, _>::into_seq_iter(path, trav).next()
            .map(|path|
                Self::match_end(&trav, PathPair::from_mode(path, query.clone(), mode))
            )
            .unwrap_or_else(|path|
                Self::index_end(trav, query.clone(), path)
            )
    }
    fn index_end(
        trav: &'a Self::Trav,
        query: FolderQuery<'a, 'g, T, D, Q, Self>,
        mut path: FolderPath<'a, 'g, T, D, Q, Self>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        path.move_width_into_start();
        Self::end_op(trav, query, Into::<StartPath>::into(path))
    }
    /// generate nodes for a child
    fn match_end(
        trav: &'a Self::Trav,
        new_paths: PathPair<FolderQuery<'a, 'g, T, D, Q, Self>, FolderPath<'a, 'g, T, D, Q, Self>>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        let (new_path, new_query) = new_paths.unpack();
        let path_next = new_path.get_end::<_, D, _>(trav);
        let query_next = new_query.get_end::<_, D, _>(trav);
        match path_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                Self::prefix_nodes(
                    trav,
                    path_next,
                    PathPair::GraphMajor(new_path, new_query),
                ),
            Ordering::Less =>
                Self::prefix_nodes(
                    trav,
                    query_next,
                    PathPair::QueryMajor(new_query, new_path),
                ), // todo: path in query
            Ordering::Equal =>
                if path_next == query_next {
                    // continue with match node
                    let mut path = new_path.clone();
                    let query = new_query.clone();
                    path.on_match::<_, D, _>(trav);
                    vec![
                        IntoSequenceIterator::<_, D, _>::into_seq_iter(query, trav).next()
                            .map(|query|
                                ToTraversalNode::match_node(
                                    path.clone(),
                                    query,
                                    new_query.clone(),
                                )
                            )
                            .unwrap_or_else(|query|
                                ToTraversalNode::end_node(Some(QueryResult::new(
                                    path.reduce_end::<_, D, _>(trav),
                                    query,
                                )))
                            )
                    ]
                } else if path_next.width == 1 {
                    vec![
                        ToTraversalNode::mismatch_node(PathPair::GraphMajor(new_path, new_query))
                    ]
                } else {
                    Self::prefix_nodes(
                        trav,
                        path_next,
                        PathPair::GraphMajor(new_path.clone(), new_query.clone()),
                    )
                    .into_iter()
                    .chain(
                        Self::prefix_nodes(
                            trav,
                            query_next,
                            PathPair::QueryMajor(new_query, new_path),
                        )
                    )
                    .collect_vec()
                }
        }
    }
    /// generate child nodes for index prefixes
    fn prefix_nodes(
        trav: &'a Self::Trav,
        index: Child,
        new_paths: PathPair<FolderQuery<'a, 'g, T, D, Q, Self>, FolderPath<'a, 'g, T, D, Q, Self>>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {

        let graph = trav.graph();
        let vertex = graph.expect_vertex_data(index);
        let mut child_patterns = vertex.get_children().into_iter().collect_vec();

        child_patterns.sort_unstable_by_key(|(_, p)| p.first().unwrap().width);
        child_patterns
            .into_iter()
            .map(|(&pid, child_pattern)| {
                let sub_index = D::head_index(child_pattern);
                let mut new_paths = new_paths.clone();
                new_paths.push_major(ChildLocation::new(index, pid, sub_index));
                Self::match_end(
                    trav,
                    new_paths,
                )
            })
            .flatten()
            .collect_vec()
    }
}
pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection, Q: TraversalQuery + 'a>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Path: TraversalPath;
    type Node: ToTraversalNode<Q, Self::Path>;
    type Break;
    type Continue;
    fn fold_found(
        trav: &'a Self::Trav,
        acc: Self::Continue,
        node: Self::Node
    ) -> ControlFlow<Self::Break, Self::Continue>;
}

pub trait TraversalQuery: AdvanceablePath + PatternStart + PatternEnd + PathFinished {}
impl<T: AdvanceablePath + PatternStart + PatternEnd + PathFinished> TraversalQuery for T {}

pub(crate) trait TraversalPath:
    AdvanceablePath +
    GraphStart +
    GraphEnd +
    From<StartPath> +
    Into<StartPath> +
    Into<GraphRangePath>
{
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> FoundPath;
    fn move_width_into_start(&mut self);
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav);
}

#[derive(Clone, Debug)]
pub(crate) enum PathPair<
    Q: TraversalQuery,
    G: TraversalPath,
> {
    GraphMajor(G, Q),
    QueryMajor(Q, G),
}
impl<
    Q: TraversalQuery,
    G: TraversalPath,
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