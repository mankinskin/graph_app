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
    type Break = (<R as ResultKind>::Indexed, Q);
    type Continue = Option<TraversalResult<<R as ResultKind>::Found, Q>>;
    //type Primer = <IndexingPolicy<'a, T, D, Q, R> as DirectedTraversalPolicy<'a, 'g, T, D, Q, R>>::Primer;

    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        let mut trav = trav.clone();
        match node {
            TraversalNode::QueryEnd(res) => {
                ControlFlow::Break((
                    R::index_found::<_, D, _>(
                        res.found,
                        &mut trav
                    ),
                    res.query,
                ))
            },
            TraversalNode::Mismatch(res) => {
                ControlFlow::Continue(search::pick_max_result::<R, _>(acc, res))
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
        self.path_indexing::<BaseResult, _, IndexingPolicy<T, D, Q, _>, Bft<_, _, _, _, _, _>>(query)
    }
    pub(crate) fn index_query_with_origin<
        Q: IndexingQuery,
    >(
        &mut self,
        query: Q,
    ) -> Result<(OriginPath<Child>, Q), NoMatch> {
        self.path_indexing::<OriginPathResult, _, IndexingPolicy<T, D, Q, _>, Bft<_, _, _, _, _, _>>(query)
    }
    fn path_indexing<
        R: ResultKind,
        Q: IndexingQuery,
        S: IndexerTraversalPolicy<'a, 'g, T, D, Q, R>,
        Ti: TraversalIterator<'a, 'g, T, D, Self, Q, S, R>,
    >(
        &'a mut self,
        query_path: Q,
    ) -> Result<(<R as ResultKind>::Indexed, Q), NoMatch> {
        let mut indexer = self.clone();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query_path.hash(&mut hasher);
        let _h = hasher.finish();
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(
                None,
                |acc, (_depth, node)|
                    <S::Folder as TraversalFolder<_, _, _, R>>::fold_found(self, acc, node)
            ) {
            ControlFlow::Continue(found) => {
                found.ok_or(NoMatch::NotFound)
                    .map(|f|
                        (
                            R::index_found::<_, D, _>(f.found, &mut indexer),
                            f.query
                        )
                    )
            }
            ControlFlow::Break((found, query)) => Ok((found, query)),
        }
    }
}