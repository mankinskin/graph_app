pub(crate) mod bft;
pub(crate) mod dft;
pub(crate) mod path;
pub(crate) mod node;
pub(crate) mod traversable;
pub(crate) mod folder;
pub(crate) mod iterator;

pub(crate) use bft::*;
pub(crate) use dft::*;
pub(crate) use path::*;
pub(crate) use node::*;
pub(crate) use traversable::*;
pub(crate) use folder::*;
pub(crate) use iterator::*;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::{
    ControlFlow,
};
use crate::{
    *,
    Child,
    ChildLocation,
    Tokenize,
    Vertexed,
    MatchDirection,
    TraversalOrder, QueryResult, Wide, PatternLocation, Pattern,
};

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

pub(crate) trait DirectedTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Q: TraversalQuery + 'a
>: Sized {

    type Trav: Traversable<'a, 'g, T>;
    type Folder: TraversalFolder<'a, 'g, T, D, Q, Trav=Self::Trav>;

    fn end_op(
        trav: &'a Self::Trav,
        query: FolderQuery<'a, 'g, T, D, Q, Self>,
        start_path: StartPath,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        Self::parent_nodes(trav, query, Some(start_path))
    }
    fn after_match_end(
        _trav: &'a Self::Trav,
        start: StartPath,
    ) -> StartPath {
        start
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
                let start_index = old_query.get_entry();
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
            .unwrap_or_else(|path| {
                Self::end_op(trav, old_query.clone(), Into::<StartPath>::into(path))
            })
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
            .unwrap_or_else(|mut path| {
                path.move_width_into_start();
                let start = Self::after_match_end(trav, Into::<StartPath>::into(path));
                Self::end_op(trav, query, start)
            })
    }
    /// generate nodes for a child
    fn match_end(
        trav: &'a Self::Trav,
        new_paths: PathPair<FolderQuery<'a, 'g, T, D, Q, Self>, FolderPath<'a, 'g, T, D, Q, Self>>,
    ) -> Vec<FolderNode<'a, 'g, T, D, Q, Self>> {
        // todo: remove "new" in current paths
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