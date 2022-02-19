use std::sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
};

use crate::{
    vertex::*,
    r#match::*,
    index::*,
    Hypergraph, HypergraphRef,
};
use itertools::*;

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
//impl<T: Tokenize, D: IndexDirection> Clone for Indexer<T, D> {
//    fn clone(&self) -> Self {
//        Self {
//            graph: self.graph.clone(),
//            _ty: Default::default(),
//        }
//    }
//}
//impl<T: Tokenize, D: IndexDirection> std::ops::Deref for Indexer<T, D> {
//    type Target = Hypergraph<T>;
//    fn deref(&self) -> &Self::Target {
//        self.graph.read().unwrap().deref()
//    }
//}
//impl<T: Tokenize, D: IndexDirection> std::ops::DerefMut for Indexer<T, D> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        self.graph.write().unwrap().deref_mut()
//    }
//}
struct Indexing<T: Tokenize, D: IndexDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<T: Tokenize, D: IndexDirection> Traversable<T> for Indexer<T, D> {
    type Node = IndexingNode;
    type State = (
        Self,
        Child,
        ChildPath,
    );
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        self.graph.read().unwrap()
    }
    fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>> {
        self.graph.write().unwrap()
    }
}
impl<'g, T: Tokenize, D: IndexDirection> BreadthFirstTraversal<'g, T> for Indexing<T, D> {
    type Trav = Indexer<T, D>;
    fn end_op(
        trav: Self::Trav,
        start_path: ChildPath,
        root: Child,
        _loc: ChildLocation,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        // root at end of parent, recurse upwards to get all next children
        //subgraph.add_index_parent(root.index, loc.parent, PatternIndex::new(loc.pattern_id, loc.sub_index));
        trav.parent_nodes(root, start_path)
    }
}
impl<'g, T: Tokenize, D: IndexDirection + 'g> DirectedTraversalPolicy<'g, T, D> for Indexing<T, D> {
}
impl<'g, T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    //pub(crate) fn index_found(
    //    &mut self,
    //    found_path: FoundPath,
    //) -> (Option<Child>, Child, Option<Child>, Pattern) {
    //    let FoundPath {
    //            root,
    //            start_path,
    //            mut end_path,
    //            remainder,
    //    } = found_path;
    //    println!("start: {:?}, end: {:?}", start_path.as_ref().map(|p| p.last().unwrap()), end_path.as_ref().map(|p| p.last().unwrap()));
    //    let left = start_path.as_ref().map(|start_path| {
    //        let mut start_path = start_path.iter();
    //        let location = start_path.next().unwrap();
    //        let inner = if let Some(end) = end_path.as_mut()
    //            .and_then(|p| p.iter_mut().find(|loc|
    //                loc.parent == location.parent && loc.pattern_id == location.pattern_id
    //            )) {
    //            let inner = self.graph_mut().index_range_in(&location.parent, location.pattern_id, location.sub_index + 1..end.sub_index).ok();
    //            end.sub_index = location.sub_index + 1;
    //            inner
    //        } else {
    //            self.index_postfix_at(&location).ok()
    //        };
    //        start_path
    //            .fold((None, inner, location), |(context, inner, prev_location), location| {
    //                let context = context.unwrap_or_else(||
    //                    self.index_pre_context_at(&prev_location).unwrap()
    //                );
    //                let context = self.index_pre_context_at(&location).map(|pre|
    //                        self.graph_mut().insert_pattern([pre, context])
    //                    )
    //                    .unwrap_or(context);
    //                let inner = self.index_post_context_at(&location).ok().map(|postfix|
    //                    if let Some(inner) = inner {
    //                        self.graph_mut().insert_pattern([inner, postfix])
    //                    } else {
    //                        postfix
    //                    }
    //                ).or(inner);
    //                if let Some(inner) = inner {
    //                    self.graph_mut().add_pattern_to_node(location.parent, [context, inner].as_slice());
    //                }
    //                (Some(context), inner, location)
    //            })
    //    });
    //    let right = end_path.map(|end_path| {
    //        let mut end_path = end_path.into_iter().rev();
    //        let location = end_path.next().unwrap();
    //        let inner = if let Some(start) = start_path.as_ref()
    //            .and_then(|p| p.iter().find(|loc|
    //                loc.parent == location.parent && loc.pattern_id == location.pattern_id
    //            )) {
    //            self.graph_mut().index_range_in(&location.parent, location.pattern_id, start.sub_index+1..location.sub_index).ok()
    //        } else {
    //            self.index_postfix_at(&location).ok()
    //        };
    //        end_path
    //            .fold((inner, None, location), |(inner, context, prev_location), location| {
    //                let context = context.unwrap_or_else(||
    //                    self.index_post_context_at(&prev_location).unwrap()
    //                );
    //                let context = self.index_post_context_at(&location).map(|post|
    //                        self.graph_mut().insert_pattern([context, post])
    //                    )
    //                    .unwrap_or(context);
    //                let inner = self.index_pre_context_at(&location).ok().map(|pre|
    //                    if let Some(inner) = inner {
    //                        self.graph_mut().insert_pattern([pre, inner])
    //                    } else {
    //                        pre
    //                    }
    //                ).or(inner);
    //                if let Some(inner) = inner {
    //                    self.graph_mut().add_pattern_to_node(location.parent, [inner, context].as_slice());
    //                }
    //                (inner, Some(context), location)
    //            })
    //    });
    //    let (lctx, inner, rctx) = match (left, right) {
    //        (None, None) => (None, root, None),
    //        (Some((lcontext, linner, _)), Some((rinner, rcontext, _))) => {
    //            let inner = self.graph_mut().insert_pattern([linner.unwrap(), rinner.unwrap()].as_slice());
    //            match (lcontext, rcontext) {
    //                (Some(lctx), Some(rctx)) => {
    //                    self.graph_mut().add_pattern_to_node(root, [lctx, inner, rctx].as_slice());
    //                }
    //                (Some(lctx), None) => {
    //                    self.graph_mut().add_pattern_to_node(root, [lctx, inner].as_slice());
    //                }
    //                (None, Some(rctx)) => {
    //                    self.graph_mut().add_pattern_to_node(root, [inner, rctx].as_slice());
    //                }
    //                (None, None) => {},
    //            };
    //            (lcontext, inner, rcontext)
    //        },
    //        (Some((lcontext, inner, _)), None) => {
    //            (lcontext, inner.unwrap(), None)
    //        }
    //        (None, Some((inner, rcontext, _))) => {
    //            (None, inner.unwrap(), rcontext)
    //        }
    //    };
    //    (lctx, inner, rctx, remainder.unwrap_or_default())
    //}
    /// includes location
    #[allow(unused)]
    pub(crate) fn index_prefix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index + 1)
    }
    /// includes location
    pub(crate) fn index_postfix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index..)
    }
    /// does not include location
    pub(crate) fn index_pre_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index)
    }
    /// does not include location
    pub(crate) fn index_post_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Result<Child, NoMatch> {
        self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index + 1..)
    }
    pub(crate) fn index_pattern<'a>(
        &'g mut self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> Result<Child, NoMatch> {
        Right::split_head_tail(pattern.as_pattern_view())
            .ok_or(NoMatch::EmptyPatterns)
            .map(|(head, tail)|
                self.index_child_in_context(head, tail)
            )
    }
    pub(crate) fn index_child_in_context(
        &'g mut self,
        head: impl AsChild,
        tail: impl IntoPattern<Item = impl AsChild>,
    ) -> Child {
        let head = head.as_child();
        if tail.is_empty() {
            head
        } else {
            self.indexing(
                head,
                tail,
            )
        }
    }
    pub(crate) fn index_split(
        &mut self,
        path: ChildPath,
    ) -> IndexedChild {
        path.into_iter().fold(None, |acc, location| {
            let context = self.index_pre_context_at(&location).ok();
            let (inner, context, location) = if let Some(IndexedChild {
                    context: prev_context,
                    inner: prev_inner,
                    ..
                }) = acc {
                let context = context.and_then(|context|
                    prev_context.map(|prev_context|
                        self.graph_mut().insert_pattern([context, prev_context].as_slice())
                    )
                    .or(Some(context))
                )
                .or(prev_context);
                let inner = self.index_post_context_at(&location).expect("Invalid child location!");
                (
                    self.graph_mut().insert_pattern([prev_inner, inner]),
                    context,
                    location,
                )
            } else {
                (
                    self.index_postfix_at(&location).expect("Invalid child location!"),
                    context,
                    location,
                )
            };
            Some(IndexedChild {
                context,
                inner,
                location,
            })
        })
        .unwrap()
    }
    /// creates an IndexingNode::Parent for each parent of root, extending its start path
    fn bft_indexing<
        'a,
        V: Vertexed<'a, 'g>,
        C: AsChild,
        Q: IntoPattern<Item = C>,
    >(
        &mut self,
        start: V,
        query: Q,
    ) -> Vec<IndexedPath> {
        let start_index = start.as_child();
        // try any parent, i.e. first
        let query = query.as_pattern_view();
        D::pattern_head(query)
            .ok_or_else(|| NoMatch::SingleIndex)
            .map(|query_next| {
                let query_next = query_next.to_child();
                // if context not empty
                // breadth first traversal
                //let subgraph = Arc::new(RwLock::new(Subgraph::new()));
                let indexer = self.clone();
                let mut indexer2 = self.clone();

                // breadth first iterator over graph from start
                Bft::new(IndexingNode::Start(start_index), |node| {
                    match node.clone() {
                        IndexingNode::Start(root) =>
                            // search parents of start
                            indexer.parent_nodes(
                                root,
                                vec![],
                            )
                            .into_iter(),
                        IndexingNode::Parent(start_path) => {
                            Indexing::successor_nodes(
                                indexer.clone(),
                                start_path,
                                query_next,
                            )
                            .into_iter()
                            // look at path segment towards start_index (entry)
                            //let ChildLocation {
                            //    parent: root,
                            //    pattern_id,
                            //    sub_index,
                            //} = start_path.last().unwrap().clone();
                            //let indexer = indexer.clone();
                            //let parent_vertex = indexer.graph().expect_vertex_data(root.index());
                            //let child_patterns = parent_vertex.get_children();
                            //// look in child pattern containing entry from start index
                            //let pattern = child_patterns.get(&pattern_id).unwrap();
                            //if let Some(next_child) = D::next_child(pattern, sub_index) {
                            //    // compare next successor
                            //    if next_child.width > query_next.width {
                            //        // might be a match, search successor
                            //        Some(IndexingNode::Child(
                            //            start_path,
                            //            root,
                            //            vec![ChildLocation::new(root, pattern_id, D::index_next(sub_index).unwrap())],
                            //            next_child,
                            //        ))
                            //    } else {
                            //        // matching or no matching children
                            //        None
                            //    }
                            //    .into_iter()
                            //    .collect_vec()
                            //} else {
                            //    // no next child, search parents
                            //    Indexing::end_op((indexer, root, start_path))
                            //}
                            //.into_iter()
                        },
                        IndexingNode::Child(start_path, root, path, child) => {
                            // compare successor
                            if child == query_next {
                                vec![IndexingNode::Child(start_path, root, path, child)].into_iter()
                            } else {
                                let graph = &*indexer.graph();
                                let vertex = graph.expect_vertex_data(child);
                                let child_patterns = vertex.get_children();
                                // search prefix of each child pattern
                                child_patterns
                                    .into_iter()
                                    .filter_map(|(&pid, pattern)| {
                                        let &head = D::pattern_head(pattern).unwrap();
                                        (head.width() > query_next.width()).then(|| {
                                            let mut path = path.clone();
                                            path.push(ChildLocation::new(child, pid, D::head_index(pattern)));
                                            IndexingNode::Child(start_path.clone(), root, path, head)
                                        })
                                    })
                                    .collect_vec()
                                    .into_iter()
                            }
                        },
                    }
                })

                // keep matching paths
                .filter_map(|(_, node)|
                    match node {
                        IndexingNode::Parent(start_path) => {
                            // look at entry from start into parent
                            let ChildLocation {
                                parent,
                                pattern_id,
                                sub_index,
                            } = *start_path.last().unwrap();
                            let (parent, next_child) = {
                                let graph = &*indexer2.graph();
                                let parent_vertex = graph.expect_vertex_data(parent.index());
                                let child_patterns = parent_vertex.get_children();
                                let pattern = child_patterns.get(&pattern_id).unwrap();
                                (
                                    parent_vertex.as_child(),
                                    D::next_child(pattern, sub_index)
                                )
                            };
                            next_child.and_then(|child_next|
                                (child_next == query_next).then(|| {
                                    // next index in parent matches query
                                    let next_index = D::index_next(sub_index).unwrap();
                                    let query_tail = D::pattern_tail(query.as_pattern_view()).into_pattern();
                                    let end_path = vec![
                                        ChildLocation::new(
                                            parent,
                                            pattern_id,
                                            next_index,
                                        )
                                    ];
                                    let indexed = indexer2.index_split(start_path);
                                    IndexedPath::new(indexed, end_path, query_tail)
                                })
                            )
                        },
                        IndexingNode::Child(start_path, _root, mut end_path, current) =>
                            {
                                let graph = &*indexer2.graph();
                                let vertex = graph.expect_vertex_data(current);
                                let child_patterns = vertex.get_children();
                                child_patterns
                                    .into_iter()
                                    .find(|(_pid, pattern)| {
                                        let &head = D::pattern_head(pattern).unwrap();
                                        head == query_next
                                    })
                                    .map(|(pid, pattern)| (*pid, pattern.clone()))
                            }
                            .map(|(pid, pattern)| {
                                // child prefix matches query
                                end_path.push(ChildLocation::new(current, pid, D::head_index(&pattern)));
                                let query_tail = D::pattern_tail(query.as_pattern_view()).into_pattern();
                                let indexed = indexer2.index_split(start_path);
                                IndexedPath::new(indexed, end_path, query_tail)
                            }),
                        _ => None,
                    }
                )
                // iterator over all roots with paths from start to query_next
                .map(|indexed_path| {
                    println!("{:#?}", indexed_path);
                    //subgraph.entry(found_path.root)
                    //    .and_modify(|indices| {
                    //        indices.insert((
                    //    })
                    let end_path = indexed_path.end_path.clone().unwrap();
                    match {
                        let graph = self.graph();
                        graph.matcher::<D>()
                            .match_path_in_context(
                                end_path,
                                indexed_path.remainder.clone().unwrap_or_default(),
                            )
                    } {
                        Ok(match_path) =>
                            match match_path.remainder {
                                // matches completely, search parents
                                GrowthRemainder::Query(remainder)
                                    // todo: pass start path
                                    => self.bft_indexing(indexed_path.indexed.location.parent, remainder.clone()),
                                // matches partially, we ar at root
                                GrowthRemainder::Child(_) => vec![IndexedPath {
                                    indexed: indexed_path.indexed,
                                    end_path: if match_path.path.is_empty() {
                                        None
                                    } else {
                                        Some(match_path.path)
                                    },
                                    remainder: None,
                                }],
                                // matches completely, and query is finished, we are at root
                                GrowthRemainder::None => vec![IndexedPath {
                                    indexed: indexed_path.indexed,
                                    end_path: if match_path.path.len() < 2 {
                                        None
                                    } else {
                                        Some(match_path.path)
                                    },
                                    remainder: None,
                                }]
                            },
                        // found mismatch, we are at root
                        Err(mismatch_path) =>
                            vec![IndexedPath {
                                indexed: indexed_path.indexed,
                                end_path: if mismatch_path.path.is_empty() {
                                    None
                                } else {
                                    Some(mismatch_path.path)
                                },
                                remainder: Some(mismatch_path.query),
                            }],
                    }
                })
                .flatten()
                .collect_vec()
            })
            //.or_else(|_| Ok(IndexedPath::remainder(indexed_path.indexed, remainder))),
            .unwrap_or_default()
    }
    fn indexing<
        'a,
        V: Vertexed<'a, 'g>,
        C: AsChild,
        Q: IntoPattern<Item = C>,
    >(
        &mut self,
        start: V,
        query: Q,
    ) -> Child {
        let start_child = start.as_child();
        self.bft_indexing(start, query)
            .into_iter()
            // collect paths into subgraph
            .fold(None, |acc: Option<IndexedPath>, indexed| {
                if let Some(acc) = acc {
                    assert!(
                        acc.indexed.location.parent == indexed.indexed.location.parent,
                        "Found multiple roots!"
                    );
                    Some(acc)
                } else {
                    Some(indexed)
                }
            })
            .map(|indexed| indexed.indexed.location.parent)
            .unwrap_or_else(|| {
                start_child // todo
            })
    }
}