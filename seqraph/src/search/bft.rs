use std::cmp::Ordering;
use std::collections::VecDeque;
use std::iter::{Extend, FusedIterator};
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
    RangePath,
    QueryRangePath,
    PatternLocation,
    IntoPatternLocation,
};

#[derive(Clone)]
pub struct Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
{
    queue: VecDeque<(usize, T)>,
    iter_children: F,
}

impl<T, F, I> Bft<T, F, I>
where
    T: Sized,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T>,
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
            let children = (self.iter_children)(&node);
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

pub(crate) trait BftNode {
    fn query_node(query: QueryRangePath) -> Self;
    fn root_node(query: QueryRangePath, start_path: StartPath) -> Self;
    fn match_node(path: RangePath, query: QueryRangePath) -> Self;
}
#[derive(Clone, Debug)]
pub struct StartPath {
    path: ChildPath,
    entry: ChildLocation,
}
pub(crate) trait Traversable<T: Tokenize> {
    type Node: BftNode;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>>;
    fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>>;
    fn parent_nodes(
        &self,
        query: QueryRangePath,
        start_path: Option<StartPath>,
    ) -> Vec<Self::Node> {
        let graph = &*self.graph();
        let vertex = if let Some(start_path) = start_path {
            start_path.entry.parent
        } else {
            query.get_entry()
        }.vertex(&graph);
        let mut parents = vertex.get_parents().into_iter().collect_vec();
        // try parents in ascending width (might not be needed in indexing)
        parents.sort_unstable_by_key(|a| a.1.width);
        parents.into_iter()
            .map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent.pattern_indices
                    .iter()
                    .map(|&pi| {
                        let root_entry = ChildLocation::new(p, pi.pattern_id, pi.sub_index);
                        let start_path = if let Some(mut start_path) = start_path {
                            let segment = start_path.entry;
                            start_path.path.push(segment);
                            start_path.entry = root_entry;
                            start_path
                        } else {
                            StartPath {
                                entry: root_entry,
                                path: vec![],
                            }
                        };
                        Self::Node::root_node(query, start_path)
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
}
pub(crate) trait BreadthFirstTraversal<'g, T: Tokenize>: Sized {
    type Trav: Traversable<T>;
    fn end_op(
        trav: Self::Trav,
        query: QueryRangePath,
        start_path: StartPath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node>;
}
pub(crate) trait DirectedTraversalPolicy<'g, T: Tokenize, D: MatchDirection>: BreadthFirstTraversal<'g, T> {
    fn new_root_path(
        trav: &<Self as BreadthFirstTraversal<'g, T>>::Trav,
        start_path: StartPath,
    ) -> Option<RangePath> {
        let StartPath {
            entry,
            path: start,
        } = start_path;
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(entry);
        Some(
            RangePath {
                entry: entry.sub_index,
                root_pattern: entry.into_pattern_location(),
                start,
                exit: D::index_next(pattern, entry.sub_index)?,
                end: vec![],
            }
        )
    }
    fn root_successor_nodes(
        trav: <Self as BreadthFirstTraversal<'g, T>>::Trav,
        query: QueryRangePath,
        start_path: StartPath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        // find parent partition with matching context
        if let Some(path) = Self::new_root_path(&trav, start_path) {
            Self::match_next(
                trav,
                path,
                query,
            )
            .into_iter()
            .collect_vec()
        } else {
            <Self as BreadthFirstTraversal<'g, T>>::end_op(trav, query, start_path)
        }
    }
    /// generate nodes for a child
    fn match_next(
        trav: <Self as BreadthFirstTraversal<'g, T>>::Trav,
        path: RangePath,
        query: QueryRangePath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        let child_next = path.get_next(trav);
        let query_next = query.get_next(trav);
        match child_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                Self::prefix_nodes(
                    trav,
                    child_next,
                    path,
                    query,
                ),
            Ordering::Less => vec![], // todo: path in query
            Ordering::Equal =>
                (child_next == query_next).then(|| {
                    // continue with match node
                    Self::successor_nodes(trav, path, query)
                })
                .into_iter()
                .flatten()
                .collect_vec(),
        }
    }
    /// generate child nodes for index prefixes
    fn prefix_nodes(
        trav: <Self as BreadthFirstTraversal<'g, T>>::Trav,
        index: Child,
        path: RangePath,
        query: QueryRangePath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        let graph = trav.graph();
        let vertex = graph.expect_vertex_data(index);
        let child_patterns = vertex.get_children();
        child_patterns
            .into_iter()
            .map(|(&pid, child_pattern)| {
                let &child_prefix = D::pattern_head(child_pattern).unwrap();
                let sub_index = D::head_index(child_pattern);
                let mut path = path.clone();
                path.push_next(ChildLocation::new(index, pid, sub_index));
                <Self::Trav as Traversable<T>>::Node::match_node(
                    path,
                    query.clone(),
                )
            })
            .collect_vec()
    }
    fn successor_nodes(
        trav: <Self as BreadthFirstTraversal<'g, T>>::Trav,
        path: RangePath,
        query: QueryRangePath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        // find parent partition with matching context
        // todo: get pattern of current node, not root
        // todo: get next child in that pattern
        if path.advance_end::<_, _, D>(trav) {
            Self::match_next(
                trav,
                path,
                query,
            )
        } else {
            <Self as BreadthFirstTraversal<'g, T>>::end_op(trav, query, path.into_start_path())
        }
    }
}