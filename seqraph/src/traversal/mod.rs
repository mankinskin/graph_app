pub(crate) mod bft;
pub(crate) mod dft;
pub(crate) mod path;

pub(crate) use bft::*;
pub(crate) use dft::*;
pub use path::*;

use std::cmp::Ordering;
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
    QueryFound, TraversalOrder,
};

#[derive(Clone, Debug)]
pub(crate) enum TraversalNode {
    Query(QueryRangePath),
    Root(QueryRangePath, Option<StartPath>, ChildLocation),
    Match(GraphRangePath, QueryRangePath, QueryRangePath),
    End(Option<QueryFound>),
    Mismatch(GraphRangePath),
}
pub(crate) trait Traversable<'a: 'g, 'g, T: Tokenize>: Sized + 'a {
    type Guard: Traversable<'g, 'g, T> + Deref<Target=Hypergraph<T>>;
    fn graph(&'a self) -> Self::Guard;
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for &'a Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'a self) -> Self::Guard {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for &'a mut Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'a self) -> Self::Guard {
        &*self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for RwLockReadGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'a self) -> Self::Guard {
        &*self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'a self) -> Self::Guard {
        &**self
    }
}

pub(crate) trait TraversableMut<'a: 'g, 'g, T: Tokenize> : Traversable<'a, 'g, T> {
    type GuardMut: TraversableMut<'g, 'g, T> + Deref<Target=Hypergraph<T>> + DerefMut;
    fn graph_mut(&'a mut self) -> Self::GuardMut;
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for Hypergraph<T> {
    type Guard = &'g Self;
    fn graph(&'a self) -> Self::Guard {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for Hypergraph<T> {
    type GuardMut = &'g mut Self;
    fn graph_mut(&'a mut self) -> Self::GuardMut {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for &'a mut Hypergraph<T> {
    type GuardMut = &'g mut Hypergraph<T>;
    fn graph_mut(&'a mut self) -> Self::GuardMut {
        *self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'g> TraversableMut<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type GuardMut = &'g mut Hypergraph<T>;
    fn graph_mut(&'a mut self) -> Self::GuardMut {
        &mut **self
    }
}
pub(crate) trait TraversalIterator<'a: 'g, 'g, T, Trav, D, S>: Iterator<Item = (usize, TraversalNode)>
where
    T: Tokenize + 'a,
    Trav: Traversable<'a, 'g, T>,
    D: MatchDirection + 'a,
    S: DirectedTraversalPolicy<'a, 'g, T, D, Trav=Trav>,
{
    fn new(trav: &'a Trav, root: TraversalNode) -> Self;
    fn iter_children(trav: &'a Trav, node: &TraversalNode) -> Vec<TraversalNode> {
        match node.clone() {
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
pub(crate) trait DirectedTraversalPolicy<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection + 'a>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Folder: TraversalFolder<'a, 'g, T, D, Trav=Self::Trav>;
    fn end_op(
        trav: &'a Self::Trav,
        query: QueryRangePath,
        start_path: StartPath,
    ) -> Vec<TraversalNode>;
    fn parent_nodes(
        trav: &'a Self::Trav,
        query: QueryRangePath,
        start: Option<StartPath>,
    ) -> Vec<TraversalNode> {

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
                TraversalNode::Root(
                    query.clone(),
                    start.clone(),
                    p,
                )
            )
            .collect_vec()
    }
    fn root_successor_nodes(
        trav: &'a Self::Trav,
        old_query: QueryRangePath,
        old_start: Option<StartPath>,
        parent_entry: ChildLocation,
    ) -> Vec<TraversalNode> {
        let start_index = old_query.get_entry();
        let graph = trav.graph();
        let pre_start = match old_start.clone() {
            Some(StartPath::First { entry, width, .. }) => {
                let pattern = graph.expect_pattern_at(entry);
                println!("first {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
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
                println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
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
                println!("start {} -> {}, {}", start_index.index, parent_entry.parent.index, start_index.width);
                StartPath::First {
                    entry: parent_entry,
                    child: start_index,
                    width: start_index.width,
                }
            }
        };
        drop(graph);
        let mut path = GraphRangePath::new(pre_start);
        if path.advance_next::<_, _, D>(trav) {
            Self::match_end(&trav, PathPair::GraphMajor(path, old_query))
        } else {
            Self::index_end(trav, old_query, path)
        }
    }
    fn query_start(
        trav: &'a Self::Trav,
        mut query: QueryRangePath,
    ) -> Vec<TraversalNode> {
        if query.advance_next::<_, _, D>(trav) {
            Self::parent_nodes(
                trav,
                query,
                None,
            )
        } else {
            vec![TraversalNode::End(None)]
        }
    }
    fn after_match(
        trav: &'a Self::Trav,
        paths: PathPair,
    ) -> Vec<TraversalNode> {
        let mode = paths.mode();
        let (mut path, query) = paths.unpack();
        if path.advance_next::<_, _, D>(trav) {
            Self::match_end(&trav, PathPair::from_mode(path, query, mode))
        } else {
            Self::index_end(trav, query, path)
        }
    }
    fn index_end(
        trav: &'a Self::Trav,
        query: QueryRangePath,
        mut path: GraphRangePath,
    ) -> Vec<TraversalNode> {
        path.move_width_into_start();
        Self::end_op(trav, query, path.into_start_path())
    }
    /// generate nodes for a child
    fn match_end(
        trav: &'a Self::Trav,
        new_paths: PathPair,
    ) -> Vec<TraversalNode> {
        let (new_path, new_query) = new_paths.unpack();
        let path_next = new_path.get_end(trav);
        let query_next = new_query.get_end(trav);
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
                    let mut query = new_query.clone();
                    path.on_match(trav);
                    vec![
                        if query.advance_next::<_, _, D>(trav) {
                            TraversalNode::Match(
                                path,
                                query,
                                new_query.clone(),
                            )
                        } else {
                            let found = QueryFound::new(
                                path.reduce_end::<_, _, D>(trav),
                                query,
                            );
                            TraversalNode::End(Some(found))
                        }
                    ]
                } else if path_next.width == 1 {
                    vec![
                        TraversalNode::Mismatch(new_path)
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
        new_paths: PathPair,
    ) -> Vec<TraversalNode> {

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
pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Break;
    type Continue;
    fn fold_found(
        trav: &'a Self::Trav,
        acc: Self::Continue,
        node: TraversalNode
    ) -> ControlFlow<Self::Break, Self::Continue>;
}
