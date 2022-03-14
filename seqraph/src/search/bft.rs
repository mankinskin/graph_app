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

#[derive(Clone, Debug)]
pub(crate) enum BftNode {
    Query(QueryRangePath),
    Root(QueryRangePath, StartPath),
    Match(RangePath, QueryRangePath),
    End(QueryFound),
    Mismatch(QueryFound),
}
//pub(crate) trait BftNode {
//    fn query_node(query: QueryRangePath) -> Self;
//    fn root_node(query: QueryRangePath, start_path: StartPath) -> Self;
//    fn match_node(path: RangePath, query: QueryRangePath) -> Self;
//    fn end_node(found: QueryFound) -> Self;
//    fn mismatch_node(found: QueryFound) -> Self;
//}
#[derive(Clone, Debug)]
pub struct StartPath {
    pub(crate) path: ChildPath,
    pub(crate) entry: ChildLocation,
    pub(crate) width: usize,
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
        trav: Self::Trav,
        query: QueryRangePath,
        start_path: StartPath,
    ) -> Vec<BftNode>;
    fn parent_nodes(
        trav: Self::Trav,
        query: QueryRangePath,
        start: Option<StartPath>,
    ) -> Vec<BftNode> {

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
                        let parent_entry = ChildLocation::new(p, pi.pattern_id, pi.sub_index);
                        let start = if let Some(mut start) = start.clone() {
                            let pattern = graph.expect_pattern_at(start.entry);
                            if start.path.is_empty() && start.entry.sub_index != D::head_index(&pattern) {
                                start.path.push(start.entry);
                            }
                            //start.width = start.width + pattern_width(D::front_context(&pattern, start.entry.sub_index));
                            start.entry = parent_entry;
                            start
                        } else {
                            StartPath {
                                entry: parent_entry,
                                path: vec![],
                                width: vertex.width,
                            }
                        };
                        BftNode::Root(
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
    ) -> Vec<BftNode> {

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
    ) -> Vec<BftNode> {

        let child_next = path.advance_next(&trav);

        let (query_next, child_next) = if let Some(query_next) = query.advance_next(&trav) {
            if let Some(child_next) = path.advance_next(&trav) {
                (query_next, child_next)
            } else {
                return Self::end_op(trav, query, path.into_start_path());
            }
        } else {
            return vec![
                BftNode::End(
                    QueryFound::new(
                        found,
                        query,
                    )
                )
            ];
        };
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
                    Self::successor_nodes(
                        trav,
                        path,
                        query,
                    )
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
    ) -> BftNode {
        let found = path.reduce_mismatch::<_, _, D>(trav);
        BftNode::Mismatch(
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
    ) -> Vec<BftNode> {

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
                BftNode::Match(
                    path,
                    query.clone(),
                )
            })
            .collect_vec()
    }
    //fn successor_nodes(
    //    trav: Self::Trav,
    //    mut path: RangePath,
    //    mut query: QueryRangePath,
    //) -> Vec<BftNode> {

    //    if query.advance_end::<_, _, D>(&trav) {
    //        if path.advance_end::<_, _, D>(&trav) {
    //            Self::match_next(
    //                trav,
    //                path,
    //                query,
    //            )
    //        } else {
    //        }
    //    } else {
    //        let found = if path.advance_end::<_, _, D>(&trav) {
    //            path.reduce_mismatch::<_, _, D>(trav)
    //        } else {
    //            path.into()
    //        };
    //        vec![
    //            BftNode::End(
    //                QueryFound::new(
    //                    found,
    //                    query,
    //                )
    //            )
    //        ]
    //    }
    //}
}

pub(crate) fn fold_found(
    acc: Option<QueryFound>,
    node: BftNode
) -> ControlFlow<QueryFound, Option<QueryFound>> {
    match node {
        BftNode::End(found) => {
            ControlFlow::Break(found)
        },
        BftNode::Mismatch(found) => {
            match &found.found {
                FoundPath::Complete(_) => {
                    //println!("found: {:?}", found);
                    if found.found.width() > acc.as_ref().map(|f| f.found.width()).unwrap_or_default() {
                        ControlFlow::Continue(Some(found))
                    } else {
                        ControlFlow::Continue(acc)
                    }
                },
                FoundPath::Range(path) => {
                    //println!("path: {:?}", path);
                    //println!("acc: {:?}", acc);
                    if path.width > acc.as_ref().map(|f| f.found.width()).unwrap_or_default() { 
                        ControlFlow::Continue(Some(found))
                    } else {
                        ControlFlow::Continue(acc)
                    }
                },
            }
        },
        _ => ControlFlow::Continue(acc)
    }
}