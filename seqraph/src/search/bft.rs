use std::cmp::Ordering;
use std::collections::VecDeque;
use std::iter::{Extend, FusedIterator};
use std::ops::ControlFlow;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use itertools::Itertools;

use crate::{
    ChildPath,
    Child,
    ChildLocation,
    Tokenize,
    Hypergraph,
    Vertexed,
    MatchDirection,
    GraphRangePath,
    QueryRangePath,
    QueryFound, PathPair,
};


#[derive(Clone)]
pub struct Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: IntoIterator<Item = T>,
{
    queue: VecDeque<(usize, T)>,
    iter_children: F,
}

impl<T, F, I> Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: IntoIterator<Item = T>,
{
    #[inline]
    pub fn new(root: T, iter_children: F) -> Self {
        Self {
            queue: VecDeque::from(vec![(0, root)]),
            iter_children,
        }
    }
}

impl<T, F, I> Iterator for Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
    type Item = (usize, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((depth, node)) = self.queue.pop_front() {
            let children = (self.iter_children)(&node).into_iter();
            self.queue.extend(children.map(|child| (depth + 1, child)));

            Some((depth, node))
        } else {
            None
        }
    }
}

impl<T, F, I> FusedIterator for Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
}

#[derive(Clone, Debug)]
pub(crate) enum BftNode {
    Query(QueryRangePath),
    Root(QueryRangePath, Option<StartPath>, ChildLocation),
    Match(GraphRangePath, QueryRangePath, QueryRangePath),
    End(Option<QueryFound>),
    Mismatch,
}
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
pub(crate) trait Traversable<T: Tokenize> {
    //type Node: BftNode;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>>;
    //fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>>;
}
pub(crate) trait TraversableMut<T: Tokenize> : Traversable<T> {
    fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>>;
}
impl <T: Tokenize, Trav: Traversable<T>> Traversable<T> for &Trav {
    //type Node = <Trav as Traversable<T>>::Node;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        Trav::graph(self)
    }
}
impl <T: Tokenize, Trav: Traversable<T>> Traversable<T> for &mut Trav {
    //type Node = <Trav as Traversable<T>>::Node;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        Trav::graph(self)
    }
}
impl <T: Tokenize, Trav: TraversableMut<T>> TraversableMut<T> for &mut Trav {
    fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>> {
        Trav::graph_mut(self)
    }
}
pub(crate) trait DirectedTraversalPolicy<'g, T: Tokenize, D: MatchDirection>: Sized {
    type Trav: Traversable<T>;
    fn end_op(
        trav: &Self::Trav,
        query: QueryRangePath,
        start_path: StartPath,
    ) -> Vec<BftNode>;
    fn parent_nodes(
        trav: Self::Trav,
        query: QueryRangePath,
        start: Option<StartPath>,
    ) -> Vec<BftNode> {

        let graph = trav.graph();
        let start_index = match start {
            Some(StartPath::First(entry, _, _)) |
            Some(StartPath::Path(entry, _, _)) =>
                entry.parent,
            None => query.get_entry()
        };
        let vertex = start_index.vertex(&graph).clone();
        let mut parents = vertex.get_parents().into_iter().collect_vec();

        // try parents in ascending width (might not be needed in indexing)
        parents.sort_unstable_by_key(|a| a.1.width);
        parents.into_iter()
            .map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent.pattern_indices
                    .iter()
                    .sorted_unstable_by_key(|pi| pi.sub_index)
                    .map(|&pi| {
                        let parent_entry = ChildLocation::new(p, pi.pattern_id, pi.sub_index);
                        BftNode::Root(
                            query.clone(),
                            start.clone(),
                            parent_entry,
                        )
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
    fn root_successor_nodes(
        trav: Self::Trav,
        old_query: QueryRangePath,
        old_start: Option<StartPath>,
        parent_entry: ChildLocation,
    ) -> Vec<BftNode> {
        let start_index = old_query.get_entry();
        let graph = trav.graph();
        let pre_start = match old_start.clone() {
            Some(StartPath::First(entry, _, width)) => {
                let pattern = graph.expect_pattern_at(entry);
                println!("first {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                StartPath::Path(parent_entry, if entry.sub_index != D::head_index(&pattern) {
                    vec![entry]
                } else {
                    vec![]
                }, width)
            },
            Some(StartPath::Path(entry, mut path, width)) => {
                println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                let pattern = graph.expect_pattern_at(entry);
                if entry.sub_index != D::head_index(&pattern) || !path.is_empty() {
                    path.push(entry);
                }
                StartPath::Path(parent_entry, path, width)
            },
            None => {
                println!("start {} -> {}, {}", start_index.index, parent_entry.parent.index, start_index.width);
                StartPath::First(
                    parent_entry,
                    start_index,
                    start_index.width,
                )
            }
        };
        let mut path = GraphRangePath::new(pre_start);
        if path.advance_next::<_, _, D>(&trav) {
            Self::match_end(&trav, PathPair::GraphMajor(path, old_query))
        } else {
            Self::end_op(&trav, old_query, path.start)
        }
    }
    fn query_start(
        trav: Self::Trav,
        mut query: QueryRangePath,
    ) -> Vec<BftNode> {
        if query.advance_next::<_, _, D>(&trav) {
            Self::parent_nodes(
                trav,
                query,
                None,
            )
        } else {
            vec![BftNode::End(None)]
        }
    }
    fn after_match(
        trav: &Self::Trav,
        paths: PathPair,
    ) -> Vec<BftNode> {
        let mode = paths.mode();
        let (mut path, query) = paths.unpack();
        if path.advance_next::<_, _, D>(&trav) {
            Self::match_end(&trav, PathPair::from_mode(path, query, mode))
        } else {
            Self::end_op(&trav, query, path.start)
        }
    }
    /// generate nodes for a child
    fn match_end(
        trav: &Self::Trav,
        new_paths: PathPair,
    ) -> Vec<BftNode> {
        let (new_path, new_query) = new_paths.unpack();
        let path_next = new_path.get_end(&trav);
        let query_next = new_query.get_end(&trav);
        match path_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                Self::prefix_nodes(
                    &trav,
                    path_next,
                    PathPair::GraphMajor(new_path, new_query),
                ),
            Ordering::Less =>
                Self::prefix_nodes(
                    &trav,
                    query_next,
                    PathPair::QueryMajor(new_query, new_path),
                ), // todo: path in query
            Ordering::Equal =>
                if path_next == query_next {
                    // continue with match node
                    let mut path = new_path.clone();
                    let mut query = new_query.clone();
                    vec![
                        if query.advance_next::<_, _, D>(&trav) {
                            path.on_match(trav);
                            BftNode::Match(
                                path,
                                query,
                                new_query.clone(),
                            )
                        } else {
                            path.on_match(trav);
                            let found = QueryFound::new(
                                //FoundPath::new::<_, _, D>(trav, path),
                                path.reduce_end::<_, _, D>(trav),
                                query,
                            );
                            BftNode::End(Some(found))
                        }
                    ]
                } else if path_next.width == 1 {
                    vec![
                        BftNode::Mismatch
                    ]
                } else {
                    Self::prefix_nodes(
                        &trav,
                        path_next,
                        PathPair::GraphMajor(new_path.clone(), new_query.clone()),
                    )
                    .into_iter()
                    .chain(
                        Self::prefix_nodes(
                            &trav,
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
        trav: &Self::Trav,
        index: Child,
        new_paths: PathPair,
    ) -> Vec<BftNode> {

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

pub(crate) fn fold_found<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(
    trav: Trav,
    acc: Option<QueryFound>,
    node: BftNode
) -> ControlFlow<Option<QueryFound>, Option<QueryFound>> {
    match node {
        BftNode::End(found) => {
            ControlFlow::Break(found)
        },
        BftNode::Match(path, _, prev_query) => {
            let found = QueryFound::new(
                path.reduce_end::<_, _, D>(trav),
                prev_query,
            );
            if acc.as_ref().map(|f| found.found.gt(&f.found)).unwrap_or(true) {
                ControlFlow::Continue(Some(found))
            } else {
                ControlFlow::Continue(acc)
            }
        }
        _ => ControlFlow::Continue(acc)
    }
}