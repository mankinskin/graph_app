use crate::{
    search::*,
    Hypergraph,
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}

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
