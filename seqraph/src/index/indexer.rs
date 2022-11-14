use std::hash::Hasher;

use crate::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for Indexer<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard {
        self.graph.read().await
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for Indexer<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.write().await
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for &'a mut Indexer<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard {
        self.graph.read().await
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for &'a mut Indexer<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.write().await
    }
}
pub(crate) struct IndexingPolicy<'a, T: Tokenize, D: IndexDirection, Q: IndexingQuery, R: ResultKind> {
    _ty: std::marker::PhantomData<(&'a T, D, Q, R)>,
}
#[async_trait]
impl<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: IndexDirection + 'a,
    Q: IndexingQuery + 'a,
    R: ResultKind + 'a,
>
DirectedTraversalPolicy<'a, 'g, T, D, Q, R> for IndexingPolicy<'a, T, D, Q, R>
{
    type Trav = Indexer<T, D>;
    type Folder = Indexer<T, D>;
    //type Primer = StartLeaf;

    #[instrument(skip(trav, primer))]
    async fn after_end_match(
        trav: &'a Self::Trav,
        primer: R::Primer,
    ) -> R::Postfix {
        let trav = trav.clone();
        let path = primer.start_match_path();
        //println!("after end match {:?}", path);
        // index postfix of match

        let match_end =
            if let Some(IndexSplitResult {
                inner: post,
                location: entry,
                ..
            }) = trav
            .pather::<IndexBack>()
            .index_primary_path::<InnerSide, _>(
                path.start_path().into_iter().chain(
                    std::iter::once(&path.entry())
                ),
                //path.get_child(),
            ).await {
                MatchEnd::Path(StartLeaf { entry, child: post, width: post.width() })
            } else {
                MatchEnd::Complete(path.entry().parent)
            };
        //println!("after end match result {:?}", match_end);
        R::into_postfix(primer, match_end)
    }
}
pub(crate) trait IndexerTraversalPolicy<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: IndexDirection + 'a,
    Q: IndexingQuery + 'a,
    R: ResultKind + 'a,
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
    Q: IndexingQuery + 'a,
    R: ResultKind + 'a,
> IndexerTraversalPolicy<'a, 'g, T, D, Q, R> for IndexingPolicy<'a, T, D, Q, R>
{}

pub(crate) trait IndexingQuery: TraversalQuery {}
impl<T: TraversalQuery> IndexingQuery for T {}

#[async_trait]
impl<'a: 'g, 'g, T, D, Q, R> TraversalFolder<'a, 'g, T, D, Q, R> for Indexer<T, D>
where 
    T: Tokenize + 'a,
    D: IndexDirection + 'a,
    Q: IndexingQuery + 'a,
    R: ResultKind + 'a,
{
    type Trav = Self;
    type Break = (<R as ResultKind>::Indexed, Q);
    type Continue = Option<TraversalResult<<R as ResultKind>::Found, Q>>;

    async fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        let mut trav = trav.clone();
        match node {
            TraversalNode::QueryEnd(res) => {
                //println!("fold query end {:#?}", res);
                ControlFlow::Break((
                    R::index_found::<_, D>(
                        res.found,
                        &mut trav
                    ).await,
                    res.query,
                ))
            },
            TraversalNode::Mismatch(res) => {
                //println!("fold mismatch {:#?}", res);
                ControlFlow::Continue(search::pick_max_result::<R, _>(acc, res))
            },
            TraversalNode::MatchEnd(postfix, query) => {
                //println!("fold match end {:#?}", postfix);
                //let found = match_end
                //    .into_range_path().into_result(query);
                //if let Some(r) = found.found.get_range() {
                //    assert!(r.get_entry_pos() != r.get_exit_pos());
                //}
                ControlFlow::Continue(search::pick_max_result::<R, _>(
                    acc,
                    <R as ResultKind>::Found::from(postfix).into_result(query)
                ))
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
    pub fn contexter<Side: IndexSide<D>>(&self) -> Contexter<T, D, Side> {
        Contexter::new(self.clone())
    }
    pub fn splitter<Side: IndexSide<D>>(&self) -> Splitter<T, D, Side> {
        Splitter::new(self.clone())
    }
    pub fn pather<Side: IndexSide<D>>(&self) -> Pather<T, D, Side> {
        Pather::new(self.clone())
    }
    pub(crate) async fn index_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        let query = query.into_pattern();
        match QueryRangePath::new_directed::<D, _>(query.borrow()) {
            Ok(query_path) => self.index_query(query_path).await,
            Err((NoMatch::SingleIndex(c), path)) => Ok((c, path)),
            Err((err, _)) => Err(err),
        }
    }
    pub(crate) async fn index_query<
        Q: IndexingQuery,
    >(
        &mut self,
        query: Q,
    ) -> Result<(Child, Q), NoMatch> {
        self.path_indexing::<BaseResult, _, IndexingPolicy<T, D, Q, _>, Bft<_, _, _, _, _, _>>(query).await
    }
    pub(crate) async fn index_query_with_origin<
        Q: IndexingQuery,
    >(
        &mut self,
        query: Q,
    ) -> Result<(OriginPath<Child>, Q), NoMatch> {
        self.path_indexing::<OriginPathResult, _, IndexingPolicy<T, D, Q, _>, Bft<_, _, _, _, _, _>>(query).await
    }
    async fn path_indexing<
        R: ResultKind + 'a,
        Q: IndexingQuery + 'a,
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

        let mut acc = ControlFlow::Continue(None);
        let mut stream = pin!(Ti::new(self, TraversalNode::query_node(query_path)));

        while let Some((_depth, node)) = stream.next().await {
            match <S::Folder as TraversalFolder<_, _, _, R>>::fold_found(self, acc.continue_value().unwrap(), node).await {
                ControlFlow::Continue(c) => {
                    acc = ControlFlow::Continue(c);
                },
                ControlFlow::Break(found) => {
                    acc = ControlFlow::Break(found);
                    break;
                },
            };
        }
        match acc {
            ControlFlow::Continue(found) => {
                match found {
                    Some(f) => Ok((
                        R::index_found::<_, D>(f.found, &mut indexer).await,
                        f.query
                    )),
                    None => Err(NoMatch::NotFound),
                }
            }
            ControlFlow::Break((found, query)) => Ok((found, query)),
        }
    }
}