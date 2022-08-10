use crate::{
    search::*,
    Hypergraph,
};
use std::{sync::RwLockReadGuard, ops::ControlFlow};

#[derive(Clone, Debug)]
pub struct Searcher<T: Tokenize, D: MatchDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection> Traversable<'a, 'g, T> for Searcher<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}

trait SearchTraversalPolicy<'a: 'g, 'g, T: Tokenize, D: MatchDirection>:
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath, Trav=Searcher<T, D>, Folder=Searcher<T, D>>
{
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection>
    SearchTraversalPolicy<'a, 'g, T, D> for AncestorSearch<T, D>
{}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection>
    SearchTraversalPolicy<'a, 'g, T, D> for ParentSearch<T, D>
{}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection>
    TraversalFolder<'a, 'g, T, D, QueryRangePath> for Searcher<T, D>
{
    type Trav = Self;
    type Break = Option<QueryFound>;
    type Continue = Option<QueryFound>;
    type Node = MatchNode;
    type Path = SearchPath;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: Self::Node,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            MatchNode::QueryEnd(found) => {
                ControlFlow::Break(found)
            },
            MatchNode::Match(path, query) => {
                let found = QueryFound::new(
                    path.reduce_end::<_, D, _>(trav),
                    query
                );
                ControlFlow::Continue(Some(
                    if let Some(acc) = acc {
                        std::cmp::max_by(
                            found,
                            acc,
                            |found, acc|
                                found.found.cmp(&acc.found)
                        )
                    } else {
                        found
                    }
                ))
            }
            _ => ControlFlow::Continue(acc)
        }
    }
}
struct AncestorSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath> for AncestorSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
}
struct ParentSearch<T: Tokenize, D: MatchDirection> {
    _ty: std::marker::PhantomData<(T, D)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: MatchDirection>
    DirectedTraversalPolicy<'a, 'g, T, D, QueryRangePath> for ParentSearch<T, D>
{
    type Trav = Searcher<T, D>;
    type Folder = Searcher<T, D>;
    fn at_index_end(
        _trav: &'a Self::Trav,
        _query: &QueryRangePath,
        _start: &MatchEnd,
    ) -> Vec<FolderNode<'a, 'g, T, D, QueryRangePath, Self>> {
        vec![]
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'g, D: MatchDirection + 'g> Searcher<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    // find largest matching direct parent
    pub(crate) fn find_pattern_parent(
        &'a self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.dft_search::<ParentSearch<T, D>, _>(
            pattern,
        )
    }
    /// find largest matching ancestor for pattern
    pub(crate) fn find_pattern_ancestor(
        &'a self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.dft_search::<AncestorSearch<T, D>, _>(
            pattern,
        )
    }
    fn dft_search<
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        self.search::<Dft<_, _, _, _, _>, S, _>(
            query,
        )
    }
    fn search<
        Ti: TraversalIterator<'a, 'g, T, Self, D, QueryRangePath, S>,
        S: SearchTraversalPolicy<'a, 'g, T, D>,
        P: IntoPattern,
    >(
        &'a self,
        query: P,
    ) -> SearchResult {
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(None, |acc: Option<QueryFound>, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => found.ok_or(NoMatch::SingleIndex)
        }
    }
}
