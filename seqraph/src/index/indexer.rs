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
//impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for &'a Indexer<T, D> {
//    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
//    fn graph(&'g self) -> Self::Guard {
//        self.graph.read().unwrap()
//    }
//}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for &'a mut Indexer<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for &'a mut Indexer<T, D> {
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
        let trav = trav.clone();
        let path = primer.start_match_path();
        //println!("after end match {:?}", path);
        // index postfix of match
        let match_end =
            if let Some((post, entry)) = trav.contexter::<IndexBack>().try_context_entry_path(
                path.entry(),
                path.start_path(),
                path.get_child(),
            ) {
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

    fn fold_found(
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
                    ),
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
    pub(crate) fn index_pattern(
        &mut self,
        query: impl IntoPattern,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        let query = query.into_pattern();
        match QueryRangePath::new_directed::<D, _>(query.borrow()) {
            Ok(query_path) => self.index_query(query_path),
            Err((NoMatch::SingleIndex(c), path)) => Ok((c, path)),
            Err((err, _)) => Err(err),
        }
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
        match Ti::new(self, TraversalNode::query_node(query_path))
            .try_fold(
                None,
                |acc, (_depth, node)|
                    <S::Folder as TraversalFolder<_, _, _, R>>::fold_found(&mut indexer, acc, node)
            ) {
            ControlFlow::Continue(found) => {
                found.ok_or(NoMatch::NotFound)
                    .map(|f|
                        (
                            R::index_found::<_, D>(f.found, &mut indexer),
                            f.query
                        )
                    )
            }
            ControlFlow::Break((found, query)) => Ok((found, query)),
        }
    }
}