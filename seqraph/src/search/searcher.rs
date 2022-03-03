use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};
use itertools::*;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
//impl<'g, T: Tokenize, D: MatchDirection> Deref for Searcher<'g, T, D> {
//    type Target = Hypergraph<T>;
//    fn deref(&self) -> &Self::Target {
//        self.graph
//    }
//}

struct AncestorSearch<'g, T: Tokenize + 'g, D: MatchDirection> {
    _ty: std::marker::PhantomData<(&'g T, D)>,
}
impl<'g, T: Tokenize + 'g, D: MatchDirection + 'g> BreadthFirstTraversal<'g, T> for AncestorSearch<'g, T, D> {
    type Trav = &'g Searcher<T, D>;
    fn end_op(
        trav: Self::Trav,
        query: QueryRangePath,
        start: StartPath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        trav.parent_nodes(query, Some(start))
    }
}
impl<'g, T: Tokenize, D: MatchDirection + 'g> DirectedTraversalPolicy<'g, T, D> for AncestorSearch<'g, T, D> {
}
struct ParentSearch<'g, T: Tokenize + 'g, D: MatchDirection> {
    _ty: std::marker::PhantomData<(&'g T, D)>,
}
impl<T: Tokenize, D: MatchDirection> Traversable<T> for Searcher<T, D> {
    type Node = SearchNode;
    fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        self.graph.read().unwrap()
    }
    //fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>> {
    //    self.graph.write().unwrap()
    //}
}
impl<'g, T: Tokenize + 'g, D: MatchDirection + 'g> BreadthFirstTraversal<'g, T> for ParentSearch<'g, T, D> {
    type Trav = &'g Searcher<T, D>;
    fn end_op(
        _trav: Self::Trav,
        _query: QueryRangePath,
        _start: StartPath,
    ) -> Vec<<Self::Trav as Traversable<T>>::Node> {
        vec![]
    }
}
impl<'g, T: Tokenize, D: MatchDirection + 'g> DirectedTraversalPolicy<'g, T, D> for ParentSearch<'g, T, D> {
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Searcher<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    // find largest matching direct parent
    pub(crate) fn find_pattern_parent<'a>(
        &'g self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> SearchResult {
        self.bft_search::<ParentSearch<'_, T, D>, _, _>(
            pattern,
        )
    }
    /// find largest matching ancestor for pattern
    pub(crate) fn find_pattern_ancestor<'a>(
        &'g self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> SearchResult {
        self.bft_search::<AncestorSearch<'g, T, D>, _, _>(
            pattern,
        )
    }
    //pub(crate) fn match_parent(
    //    &self,
    //    mut start_path: ChildPath,
    //    query_next: Child,
    //    query: impl IntoPattern<Item=impl AsChild>
    //) -> Option<FoundPath> {
    //    // find next child equal to next context index
    //    let loc@ChildLocation {
    //        parent,
    //        pattern_id,
    //        sub_index,
    //    } = start_path.pop().unwrap();
    //    let graph = self.graph();
    //    let parent_vertex = graph.expect_vertex_data(parent.index());
    //    let child_patterns = parent_vertex.get_children();
    //    let pattern = child_patterns.get(&pattern_id).unwrap();
    //    D::next_child(pattern, sub_index)
    //        .and_then(|child_next|
    //            (child_next == query_next).then(|| {
    //                // todo: explain this
    //                if sub_index != D::head_index(pattern) {
    //                    start_path.push(loc);
    //                }
    //                let next_index = D::index_next(sub_index).unwrap();
    //                let query_tail = D::pattern_tail(query.as_pattern_view()).into_pattern();
    //                let end_path = vec![
    //                    ChildLocation::new(
    //                        parent_vertex.as_child(),
    //                        pattern_id,
    //                        next_index,
    //                    )
    //                ];
    //                FoundPath::new(parent, start_path, end_path, query_tail)
    //            })
    //        )
    //}
    //pub(crate) fn match_child(
    //    &self,
    //    start_path: ChildPath,
    //    root: Child,
    //    mut path: ChildPath,
    //    current: Child,
    //    query_next: Child,
    //    query: impl IntoPattern<Item=impl AsChild>
    //) -> Option<FoundPath> {
    //    // find child starting with next_child
    //    let graph = self.graph();
    //    let vertex = graph.expect_vertex_data(current);
    //    let child_patterns = vertex.get_children();
    //    child_patterns
    //        .into_iter()
    //        .find(|(_pid, pattern)| {
    //            let &head = D::pattern_head(pattern).unwrap();
    //            head == query_next
    //        })
    //        .map(|(&pid, pattern)| {
    //            path.push(ChildLocation::new(current, pid, D::head_index(pattern)));
    //            let query_tail = D::pattern_tail(query.as_pattern_view()).into_pattern();
    //            FoundPath::new(root, start_path, path, query_tail)
    //        })
    //}
    //pub(crate) fn expand_found<
    //    S: DirectedTraversalPolicy<'g, T, D, Trav=Self>,
    //>(
    //    &'g self,
    //    found_path: FoundPath,
    //) -> SearchResult {
    //    if let Some(end_path) = found_path.end_path.clone() {
    //        match self.graph().matcher::<D>()
    //            .match_path_in_context(
    //                end_path,
    //                found_path.remainder.clone().unwrap_or_default(),
    //            ) {
    //                Err(mismatch_path) =>
    //                    Ok(FoundPath {
    //                        root: found_path.root,
    //                        start_path: found_path.start_path,
    //                        end_path: if mismatch_path.path.is_empty() {
    //                            None
    //                        } else {
    //                            Some(mismatch_path.path)
    //                        },
    //                        remainder: Some(mismatch_path.query),
    //                    }),
    //                Ok(match_path) =>
    //                    match match_path.remainder {
    //                        GrowthRemainder::Query(remainder)
    //                            => self.bft_search::<S, _, _, _>(remainder.clone())
    //                                    .or_else(|_| Ok(FoundPath::remainder(found_path.root, remainder))),
    //                        GrowthRemainder::Child(_) => Ok(FoundPath {
    //                            root: found_path.root,
    //                            start_path: found_path.start_path,
    //                            end_path: if match_path.path.is_empty() {
    //                                None
    //                            } else {
    //                                Some(match_path.path)
    //                            },
    //                            remainder: None,
    //                        }),
    //                        GrowthRemainder::None => Ok(FoundPath {
    //                            root: found_path.root,
    //                            start_path: found_path.start_path,
    //                            end_path: if match_path.path.len() < 2 {
    //                                None
    //                            } else {
    //                                Some(match_path.path)
    //                            },
    //                            remainder: None,
    //                        })
    //                    },
    //            }
    //    } else {
    //        Ok(found_path)
    //    }
    //}
    fn bft_search<
        'a,
        S: DirectedTraversalPolicy<'g, T, D, Trav=&'g Self>,
        C: AsChild,
        Q: IntoPattern<Item = C>,
    >(
        &'g self,
        query: Q,
    ) -> SearchResult {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _, _>(query.as_pattern_view())?;
        Bft::new(SearchNode::Query(query_path), move |node| {
            match node.clone() {
                SearchNode::Query(query) =>
                    self.parent_nodes(
                        query,
                        None,
                    )
                ,
                SearchNode::Root(query, start_path) =>
                    S::root_successor_nodes(
                        self,
                        query,
                        start_path,
                    ),
                SearchNode::Match(path, query) =>
                    S::match_next(
                        self,
                        path,
                        query,
                    ),
                _ => vec![],
            }.into_iter()
        })
        .find_map(|(_, node)|
            match node {
                SearchNode::End(path, query) =>
                    Some(Ok(FoundPath {
                        path,
                        query,
                    })),
                _ => None,
            }
        )
        .unwrap_or_else(||
            Err(NoMatch::NotFound(query))
        )
    }
}
