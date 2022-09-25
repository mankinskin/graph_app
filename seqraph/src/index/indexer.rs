use std::hash::Hasher;

use crate::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for Indexer<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for Indexer<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.write().unwrap()
    }
}
pub(crate) struct IndexingPolicy<'a, T: Tokenize, D: IndexDirection, Q: IndexingQuery, R: ResultKind> {
    _ty: std::marker::PhantomData<(&'a T, D, Q, R)>,
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: IndexDirection + 'a,
    Q: IndexingQuery,
    R: ResultKind,
>
DirectedTraversalPolicy<'a, 'g, T, D, Q, R> for IndexingPolicy<'a, T, D, Q, R>
{
    type Trav = Indexer<T, D>;
    type Folder = Indexer<T, D>;
    //type Primer = StartLeaf;

    fn after_end_match(
        trav: &'a Self::Trav,
        primer: R::Primer,
    ) -> R::Postfix {
        let mut ltrav = trav.clone();
        let entry = primer.get_entry_location();
        // index postfix of match
        let match_end = if let Some(IndexSplitResult {
            inner: post,
            location: entry,
            ..
        }) = IndexSplit::<_, D, IndexBack>::entry_perfect_split(
            &mut ltrav,
            entry,
        ) {
            MatchEnd::Path(StartLeaf { entry, child: post, width: primer.width() })
        } else {
            MatchEnd::Complete(entry.parent)
        };
        R::into_postfix(primer, match_end)
    }
}
pub(crate) trait IndexerTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Q: IndexingQuery,
    R: ResultKind,
>:
    DirectedTraversalPolicy<
        'a, 'g, T, D, Q, R,
        Trav=Indexer<T, D>,
        Folder=Indexer<T, D>,
        //Primer = StartLeaf
    >
{
}
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: IndexDirection + 'a,
    Q: IndexingQuery,
    R: ResultKind,
> IndexerTraversalPolicy<'a, 'g, T, D, Q, R> for IndexingPolicy<'a, T, D, Q, R>
{}

pub(crate) trait IndexingQuery: TraversalQuery {}
impl<T: TraversalQuery> IndexingQuery for T {}

impl<'a: 'g, 'g, T, D, Q, R> TraversalFolder<'a, 'g, T, D, Q, R> for Indexer<T, D>
where 
    T: Tokenize + 'a,
    D: IndexDirection + 'a,
    Q: IndexingQuery,
    R: ResultKind
{
    type Trav = Self;
    type Break = (Child, Q);
    type Continue = Option<TraversalResult<<R as ResultKind>::Found, Q>>;
    //type Primer = <IndexingPolicy<'a, T, D, Q, R> as DirectedTraversalPolicy<'a, 'g, T, D, Q, R>>::Primer;

    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        let mut trav = trav.clone();
        match node {
            TraversalNode::QueryEnd(found) => {
                ControlFlow::Break((
                    Indexing::<_, D>::index_found(&mut trav, found.found.into_range_path().into()),
                    found.query,
                ))
            },
            TraversalNode::Mismatch(found) => {
                ControlFlow::Continue(search::pick_max_result::<R, _>(acc, found))
            },
            TraversalNode::MatchEnd(postfix, query) => {
                //let found = match_end
                //    .into_range_path().into_result(query);
                //if let Some(r) = found.found.get_range() {
                //    assert!(r.get_entry_pos() != r.get_exit_pos());
                //}
                ControlFlow::Continue(search::pick_max_result::<R, _>(acc, <R as ResultKind>::Found::from(postfix).into_result(query)))
            },
            _ => ControlFlow::Continue(acc)
        }
    }
}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn index_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;
        self.index_query(query_path)
    }
    pub(crate) fn index_query<
        Q: IndexingQuery,
    >(
        &mut self,
        query: Q,
    ) -> Result<(Child, Q), NoMatch> {
        self.path_indexing::<_, IndexingPolicy<T, D, Q, BaseResult>, Bft<_, _, _, _, _, _>>(query)
    }
    //pub(crate) fn find_index_path<
    //    Q: IndexingQuery,
    //>(
    //    &mut self,
    //    query: Q,
    //) -> Result<TraversalResult<Q>, NoMatch> {
    //    self.index_path_search::<_, IndexingPolicy<T, D, Q>, Bft<_, _, _, _, _>>(query)
    //        .map(|r| match r {
    //            ControlFlow::Continue(f) => f,
    //            ControlFlow::Break((found, query)) => TraversalResult::new(FoundPath::Complete(found), query),
    //        })
    //}
    pub(crate) fn index_path_search<
        Q: IndexingQuery,
        S: IndexerTraversalPolicy<'a, 'g, T, D, Q, BaseResult>,
        Ti: TraversalIterator<'a, 'g, T, D, Self, Q, S, BaseResult>,
    >(
        &'a self,
        query_path: Q,
    ) -> Result<ControlFlow<(Child, Q), TraversalResult<<BaseResult as ResultKind>::Found, Q>>, NoMatch> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query_path.hash(&mut hasher);
        let _h = hasher.finish();
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(
                None,
                |acc, (_depth, node)|
                    <S::Folder as TraversalFolder<_, _, _, BaseResult>>::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(found) =>
                found.map(ControlFlow::Continue).ok_or(NoMatch::NotFound),
            ControlFlow::Break((found, query)) => Ok(ControlFlow::Break((found, query)))
        }
    }
    fn path_indexing<
        Q: IndexingQuery,
        S: IndexerTraversalPolicy<'a, 'g, T, D, Q, BaseResult>,
        Ti: TraversalIterator<'a, 'g, T, D, Self, Q, S, BaseResult>,
    >(
        &'a mut self,
        query_path: Q,
    ) -> Result<(Child, Q), NoMatch> {
        let mut indexer = self.clone();
        match self.index_path_search::<_, S, Ti>(query_path) {
            Ok(ControlFlow::Continue(f)) =>
                Ok((Indexing::<_, D>::index_found(&mut indexer, f.found), f.query)),
            Ok(ControlFlow::Break((found, query))) => Ok((found, query)),
            Err(err) => Err(err)
        }
    }
}