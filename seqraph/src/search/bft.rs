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
    FoundPath, QueryFound, PathPair, pattern_width,
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
    Match(GraphRangePath, QueryRangePath),
    End(QueryFound),
    Mismatch(QueryFound),
}
//pub(crate) trait BftNode {
//    fn query_node(query: QueryRangePath) -> Self;
//    fn root_node(query: QueryRangePath, start_path: StartPath) -> Self;
//    fn match_node(path: GraphRangePath, query: QueryRangePath) -> Self;
//    fn end_node(found: QueryFound) -> Self;
//    fn mismatch_node(found: QueryFound) -> Self;
//}
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
    pub fn is_complete(&self) -> bool {
        // todo: file bug, && behind match not recognized as AND
        // todo: respect match direction (need graph access
        let e = match self {
            Self::Path(_, path, _) => path.is_empty(),
            _ => true,
        };
        e && self.entry().sub_index == 0
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
        query: QueryRangePath,
        old_start: Option<StartPath>,
        parent_entry: ChildLocation,
    ) -> Vec<BftNode> {
        let start_index = query.get_entry();
        let graph = trav.graph();
        let new_start = match old_start.clone() {
            Some(StartPath::First(entry, _, width)) => {
                //let pattern = graph.expect_pattern_at(entry);
                //let width = width + pattern_width(D::front_context(&pattern, entry.sub_index));
                println!("first {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                StartPath::Path(parent_entry, vec![entry], width)
            },
            Some(StartPath::Path(entry, mut path, mut width)) => {
                println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
                let pattern = graph.expect_pattern_at(entry);
                //width = width + pattern_width(D::front_context(&pattern, entry.sub_index));
                if !(entry.sub_index == D::head_index(&pattern) && path.is_empty()) {
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
        let old_found = if let Some(old_start) = old_start {
            GraphRangePath::new(old_start).into()
        } else {
            FoundPath::Complete(start_index)
        };
        let old_match = QueryFound::new(
            old_found,
            query.clone(),
        );
        match Self::advance_paths(
            &trav,
            GraphRangePath::new(new_start),
            query.clone(),
        ) {
            Ok((new_path, new_query)) => {
                Self::match_end(&trav, PathPair::GraphMajor(new_path, new_query), &old_match)
            },
            Err(up) => {
                if up {
                    Self::end_op(&trav, query, old_found.into_start_path())
                } else {
                    vec![
                        BftNode::End(old_match)
                    ]
                }
            },
        }
    }
    fn advance_paths(
        trav: &Self::Trav,
        old_path: GraphRangePath,
        old_query: QueryRangePath,
    ) -> Result<(GraphRangePath, QueryRangePath), bool> {
        let mut query = old_query.clone();
        let mut path = old_path.clone();
        if query.advance_next::<_, _, D>(&trav) {
            if path.advance_next::<_, _, D>(&trav) {
                Ok((path, query))
            } else {
                Err(true)
            }
        } else {
            Err(false)
        }
    }
    fn advance(
        trav: &Self::Trav,
        old_paths: PathPair,
    ) -> Result<PathPair, Vec<BftNode>> {
        let mode = old_paths.mode();
        let (old_path, old_query) = old_paths.unpack();
        Self::advance_paths(trav, old_path, old_query)
            .map(|(p, q)| PathPair::from_mode(p, q, mode))
    }
    fn after_match(
        trav: &Self::Trav,
        old_paths: PathPair,
    ) -> Vec<BftNode> {
        let new_paths = match Self::advance(&trav, old_paths.clone()) {
            Ok(new_paths) => new_paths,
            Err(nodes) => return nodes,
        };
        let (old_path, old_query) = old_paths.unpack();
        Self::match_end(&trav, new_paths, &QueryFound::new(old_path, old_query))
    }
    /// generate nodes for a child
    fn match_end(
        trav: &Self::Trav,
        new_paths: PathPair,
        old_match: &QueryFound,
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
                    old_match,
                ),
            Ordering::Less =>
                Self::prefix_nodes(
                    &trav,
                    query_next,
                    PathPair::QueryMajor(new_query, new_path),
                    old_match,
                ), // todo: path in query
            Ordering::Equal =>
                if path_next == query_next {
                    // continue with match node
                    vec![
                        BftNode::Match(
                            new_path.clone(),
                            new_query.clone(),
                        )
                    ]
                } else if path_next.width == 1 {
                    // todo: find matching prefixes
                    vec![
                        BftNode::Mismatch(old_match.clone())
                    ]
                } else {
                    Self::prefix_nodes(
                        &trav,
                        path_next,
                        PathPair::GraphMajor(new_path.clone(), new_query.clone()),
                        old_match,
                    )
                    .into_iter()
                    .chain(
                        Self::prefix_nodes(
                            &trav,
                            query_next,
                            PathPair::QueryMajor(new_query, new_path),
                            old_match,
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
        old_match: &QueryFound,
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
                    old_match,
                )
            })
            .flatten()
            .collect_vec()
    }
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
                    if path.start.width() > acc.as_ref().map(|f| f.found.width()).unwrap_or_default() { 
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