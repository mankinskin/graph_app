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
    IntoPatternLocation, pattern_width, FoundPath, QueryFound,
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

pub(crate) trait BftNode {
    fn query_node(query: QueryRangePath) -> Self;
    fn root_node(query: QueryRangePath, start_path: StartPath) -> Self;
    fn match_node(path: RangePath, query: QueryRangePath) -> Self;
    fn end_node(found: QueryFound) -> Self;
    fn mismatch_node(found: QueryFound) -> Self;
}
#[derive(Clone, Debug)]
pub struct StartPath {
    pub(crate) path: ChildPath,
    pub(crate) entry: ChildLocation,
    pub(crate) width: usize,
}
pub(crate) trait Traversable<T: Tokenize> {
    type Node: BftNode;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>>;
    //fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>>;
}
impl <T: Tokenize, Trav: Traversable<T>> Traversable<T> for &Trav {
    type Node = <Trav as Traversable<T>>::Node;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        Trav::graph(self)
    }
}
pub(crate) trait DirectedTraversalPolicy<'g, T: Tokenize, D: MatchDirection>: Sized {
    type Trav: Traversable<T>;
    fn end_op(
        trav: Self::Trav,
        query: QueryRangePath,
        start_path: StartPath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node>;
    fn parent_nodes(
        trav: Self::Trav,
        query: QueryRangePath,
        start: Option<StartPath>,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {

        let graph = trav.graph();
        let vertex = if let Some(start) = &start {
            start.entry.parent
        } else {
            query.get_entry()
        }.vertex(&graph).clone();
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
                        let root_entry = ChildLocation::new(p, pi.pattern_id, pi.sub_index);
                        let start = if let Some(mut start) = start.clone() {
                            if start.path.is_empty() && start.entry.sub_index != 0 {
                                start.path.push(start.entry);
                            }
                            let pattern = graph.expect_pattern_at(start.entry);
                            start.width = start.width + pattern_width(D::front_context(&pattern, start.entry.sub_index));
                            start.entry = root_entry;
                            start
                        } else {
                            StartPath {
                                entry: root_entry,
                                path: vec![],
                                width: vertex.width,
                            }
                        };
                        <Self::Trav as Traversable<T>>::Node::root_node(
                            query.clone(),
                            start,
                        )
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
    fn new_root_path(
        trav: &Self::Trav,
        start: &StartPath,
    ) -> Option<RangePath> {

        let StartPath {
            entry,
            path,
            width,
        } = start;
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(entry);

        Some(
            RangePath {
                exit: D::index_next(pattern, entry.sub_index)?,
                entry: entry.sub_index,
                root_pattern: entry.into_pattern_location(),
                start: path.clone(),
                end: vec![],
                width: *width,
            }
        )
    }
    fn root_successor_nodes(
        trav: Self::Trav,
        query: QueryRangePath,
        start: StartPath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {

        // find parent partition with matching context
        if let Some(path) = Self::new_root_path(&trav, &start) {
            Self::match_next(
                trav,
                path,
                query,
            )
            .into_iter()
            .collect_vec()
        } else {
            Self::end_op(trav, query, start)
        }
    }
    /// generate nodes for a child
    fn match_next(
        trav: Self::Trav,
        path: RangePath,
        query: QueryRangePath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {

        let child_next = path.get_next(&trav);
        let query_next = query.get_next(&trav);

        match child_next.width.cmp(&query_next.width) {
            Ordering::Greater =>
                // continue in prefix of child
                Self::prefix_nodes(
                    &trav,
                    child_next,
                    &path,
                    &query,
                ),
            Ordering::Less =>
                Self::prefix_nodes(
                    &trav,
                    query_next,
                    &path,
                    &query,
                ), // todo: path in query
            Ordering::Equal =>
                if child_next == query_next {
                    // continue with match node
                    Self::successor_nodes(trav, path, query)
                } else if child_next.width == 1{
                    // todo: find matching prefixes
                    vec![
                        Self::mismatch_node(
                            &trav,
                            path,
                            query,
                        )
                    ]
                } else {
                    Self::prefix_nodes(
                        &trav,
                        child_next,
                        &path,
                        &query,
                    )
                    .into_iter()
                    .chain(
                        Self::prefix_nodes(
                            &trav,
                            query_next,
                            &path,
                            &query,
                        )
                    )
                    .collect_vec()
                }
        }
    }
    fn mismatch_node(
        trav: &Self::Trav,
        path: RangePath,
        query: QueryRangePath,
    ) -> <Self::Trav as Traversable<T>>::Node {
        let found = path.reduce_end::<_, _, D>(trav);
        <Self::Trav as Traversable<T>>::Node::mismatch_node(
            QueryFound::new(
                found,
                query,
            )
        )
    }
    /// generate child nodes for index prefixes
    fn prefix_nodes(
        trav: &Self::Trav,
        index: Child,
        path: &RangePath,
        query: &QueryRangePath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {

        let graph = trav.graph();
        let vertex = graph.expect_vertex_data(index);
        let mut child_patterns = vertex.get_children().into_iter().collect_vec();

        child_patterns.sort_unstable_by_key(|(_, p)| p.first().unwrap().width);
        child_patterns
            .into_iter()
            .map(|(&pid, child_pattern)| {
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
        trav: Self::Trav,
        mut path: RangePath,
        mut query: QueryRangePath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {

        // find parent partition with matching context
        // todo: get pattern of current node, not root
        // todo: get next child in that pattern
        let adv_path = path.advance_end::<_, _, D>(&trav);

        if query.advance_end::<_, _, D>(&trav) {
            if adv_path {
                Self::match_next(
                    trav,
                    path,
                    query,
                )
            } else {
                Self::end_op(trav, query, path.into_start_path())
            }
        } else {
            vec![
                <Self::Trav as Traversable<T>>::Node::end_node(
                    QueryFound::new(
                        path,
                        query,
                    )
                )
            ]
        }
    }
}