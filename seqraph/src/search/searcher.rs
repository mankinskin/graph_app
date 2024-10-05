//use rayon::iter::{
//    ParallelBridge,
//    ParallelIterator,
//};
use crate::{
    search::NoMatch,
    traversal::{
        cache::state::parent::ParentState,
        folder::TraversalFolder,
        iterator::traverser::bft::Bft,
        policy::DirectedTraversalPolicy,
        result::TraversalResult,
        traversable::Traversable,
    },
};
//use rayon::iter::{
//    ParallelBridge,
//    ParallelIterator,
//};
use crate::graph::vertex::pattern::IntoPattern;

#[derive(Clone, Debug)]
pub struct Searcher<T: Traversable> {
    pub graph: T,
}

pub trait SearchTraversalPolicy<T: Traversable>:
    DirectedTraversalPolicy<Trav = Searcher<T>>
{
}

impl<T: Traversable> SearchTraversalPolicy<T> for (T, AncestorSearch) {}

impl<T: Traversable> SearchTraversalPolicy<T> for (T, ParentSearch) {}

impl<T: Traversable> TraversalFolder for Searcher<T> {
    type Iterator<'a> = Bft<'a, Self, (T, AncestorSearch)> where T: 'a;
}

#[derive(Debug)]
pub struct AncestorSearch {}

impl<T: Traversable> DirectedTraversalPolicy for (T, AncestorSearch) {
    type Trav = Searcher<T>;
}

#[derive(Debug)]
pub struct ParentSearch {}

impl<T: Traversable> DirectedTraversalPolicy for (T, ParentSearch) {
    type Trav = Searcher<T>;
    fn next_parents(
        _trav: &Self::Trav,
        _state: &ParentState,
    ) -> Vec<ParentState> {
        vec![]
    }
}

pub type SearchResult = Result<TraversalResult, NoMatch>;

impl<T: Traversable> Searcher<T> {
    pub fn new(graph: T) -> Self {
        Self { graph }
    }
    // find largest matching direct parent
    pub fn find_pattern_parent(
        &self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.bft_search::<(T, AncestorSearch), _>(pattern)
    }
    /// find largest matching ancestor for pattern
    pub fn find_pattern_ancestor(
        &self,
        pattern: impl IntoPattern,
    ) -> SearchResult {
        self.bft_search::<(T, AncestorSearch), _>(pattern)
    }
    fn bft_search<S: SearchTraversalPolicy<T>, P: IntoPattern>(
        &self,
        query: P,
    ) -> SearchResult {
        self.search(query)
    }
    //, Ti: TraversalIterator<'a, Trav = Self>
    fn search<P: IntoPattern>(
        &self,
        query: P,
    ) -> SearchResult {
        <Self as TraversalFolder>::fold_query(self, query).map_err(|(nm, _)| nm)
    }
    //#[allow(unused)]
    //fn par_search<
    //    Ti: TraversalIterator<T, D, Self, QueryRangePath, S, BaseResult> + Send,
    //    S: SearchTraversalPolicy<T, D>,
    //    P: IntoPattern,
    //>(
    //    &'a self,
    //    query: P,
    //) -> SearchResult {
    //    let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())
    //        .map_err(|(err, _)| err)?;
    //    match ParallelIterator::simplify(
    //        Ti::new(self, TraversalState::query_node(query_path))
    //            .par_bridge()
    //            .try_fold_with(None, |acc, (_depth, node)|
    //                tokio::runtime::Handle::current().block_on(
    //                    <S::Folder as TraversalFolder<_, _, _, BaseResult>>::fold_found(self, acc, node)
    //                )
    //            ),
    //            || ControlFlow::Continue(None), |a, b|
    //                match (a, b) {
    //                    (ControlFlow::Break(b), ControlFlow::Continue(_)) |
    //                    (ControlFlow::Continue(_), ControlFlow::Break(b)) =>
    //                        ControlFlow::Break(b),
    //                    (ControlFlow::Break(a), ControlFlow::Break(b)) =>
    //                        ControlFlow::Break(
    //                            std::cmp::max_by(
    //                                a,
    //                                b,
    //                                |a, b|
    //                                    a.found.cmp(&b.found)
    //                            )
    //                        ),
    //                    (ControlFlow::Continue(a), ControlFlow::Continue(b)) =>
    //                        ControlFlow::Continue(match (a, b) {
    //                            (None, None) => None,
    //                            (None, Some(found)) |
    //                            (Some(found), None) => Some(found),
    //                            (Some(a), Some(b)) =>
    //                                Some(std::cmp::max_by(a, b, |a, b| a.found.cmp(&b.found)))
    //                        })
    //                }
    //        )
    //    {
    //        ControlFlow::Continue(None) => Err(NoMatch::NotFound),
    //        ControlFlow::Continue(Some(found)) |
    //        ControlFlow::Break(found) => Ok(found)
    //    }
    //}
}
