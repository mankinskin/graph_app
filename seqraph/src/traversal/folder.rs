use crate::*;
use super::*;

use std::ops::ControlFlow;

pub(crate) type Folder<'a, 'g, T, D, Q, R, Ty>
    = <Ty as DirectedTraversalPolicy<'a, 'g, T, D, Q, R>>::Folder;

pub(crate) trait FolderQ<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
> {
    type Query: TraversalQuery;
}

impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    Ty: TraversalFolder<'a, 'g, T, D, Q, R>,
> FolderQ<'a, 'g, T, D, Q, R> for Ty {
    type Query = Q;
}

pub(crate) type FolderQuery<'a, 'g, T, D, Q, R, Ty>
    = <Folder<'a, 'g, T, D, Q, R, Ty> as FolderQ<'a, 'g, T, D, Q, R>>::Query;

pub(crate) type FolderPathPair<'a, 'g, T, D, Q, R, Ty>
    = PathPair<<R as ResultKind>::Advanced, FolderQuery<'a, 'g, T, D, Q, R, Ty>>;

#[async_trait]
pub(crate) trait ResultKind: Eq + Clone + Debug + Send + Sync + 'static + Unpin {
    type Found: Found<Self>;
    type Primer: PathPrimer<Self>;
    type Postfix: Postfix + PathAppend<Result=Self::Primer> + From<Self::Primer>;
    type Advanced: Advanced + PathPop<Result=Self::Postfix> + From<Self::Primer>;
    type Indexed: Send + Sync;
    //type Result: From<Self::Found>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<StartLeaf>) -> Self::Postfix;
    async fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<'a, 'g, T>,
    >(found: Self::Found, indexer: &'a mut Indexer<T, D>) -> Self::Indexed;
}
pub(crate) trait Found<R: ResultKind>
    : RangePath
    + FromAdvanced<<R as ResultKind>::Advanced>
    + From<<R as ResultKind>::Postfix>
    + Wide
    + Ord
    + Send
    + Sync
    + Unpin
{
}
impl<
    R: ResultKind,
    T: RangePath
    + FromAdvanced<<R as ResultKind>::Advanced>
    + From<<R as ResultKind>::Postfix>
    + Wide
    + Ord
    + Send
    + Sync
    + Unpin
> Found<R> for T {
}
pub(crate) trait PathPrimer<R: ResultKind>:
    NodePath
    + HasStartMatchPath
    + GraphEntry
    + PathAppend
    + From<StartLeaf>
    + From<R::Advanced>
    + Wide
    + Send
    + Sync
    + Unpin
{
}
impl<
    R: ResultKind,
    T: NodePath
    + HasStartMatchPath
    + GraphEntry
    + PathAppend
    + PathAppend
    + From<StartLeaf>
    + From<<R as ResultKind>::Advanced>
    + Wide
    + Send
    + Sync
    + Unpin
> PathPrimer<R> for T
{
}

pub(crate) trait Postfix: NodePath + PathReduce + IntoRangePath
    + Send + Sync
{
    fn new_complete(child: Child, origin: StartPath) -> Self;
    fn new_path(start: impl Into<StartPath>, origin: StartPath) -> Self;
}
impl<P: MatchEndPath + PathPop<Result=Self>> Postfix for MatchEnd<P> {
    fn new_complete(c: Child, _origin: StartPath) -> Self {
        Self::Complete(c)
    }
    fn new_path(start: impl Into<StartPath>, _origin: StartPath) -> Self {
        Self::Path(P::from(start.into()))
    }
}
impl<P: Postfix + RangePath> Postfix for OriginPath<P> {
    fn new_complete(c: Child, origin: StartPath) -> Self {
        Self {
            postfix: P::new_complete(c, origin.clone()),
            origin: MatchEnd::Path(origin),
        }
    }
    fn new_path(start: impl Into<StartPath>, origin: StartPath) -> Self {
        Self {
            postfix: P::new_path(start, origin.clone()),
            origin: MatchEnd::Path(origin),
        }
    }
}
pub(crate) trait Advanced:
    RootChild
    + NewAdvanced
    + HasStartMatchPath
    + HasEndMatchPath
    + HasEndPath
    + GraphEntry
    + PathComplete
    + AddMatchWidth
    + Eq
    + Clone
    + Debug
    + Send
    + Sync
    + 'static
{
}
impl<
    T: RootChild
    + NewAdvanced
    + HasStartMatchPath
    + HasEndMatchPath
    + HasEndPath
    + GraphEntry
    + PathComplete
    + AddMatchWidth
    + Eq
    + Clone
    + Debug
    + Send
    + Sync
    + 'static
> Advanced for T {
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct BaseResult;

#[async_trait]
impl ResultKind for BaseResult {
    type Found = FoundPath;
    type Primer = StartPath;
    type Postfix = MatchEnd<StartPath>;
    type Advanced = SearchPath;
    type Indexed = Child;
    fn into_postfix(_primer: Self::Primer, match_end: MatchEnd<StartLeaf>) -> Self::Postfix {
        match_end.into()
    }
    async fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<'a, 'g, T>,
    >(found: Self::Found, indexer: &'a mut Indexer<T, D>) -> Self::Indexed {
        indexer.index_found(found.into_range_path().into()).await
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct OriginPathResult;

#[async_trait]
impl ResultKind for OriginPathResult {
    type Found = OriginPath<FoundPath>;
    type Primer = OriginPath<StartPath>;
    type Postfix = OriginPath<MatchEnd<StartPath>>;
    type Advanced = OriginPath<SearchPath>;
    type Indexed = OriginPath<Child>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<StartLeaf>) -> Self::Postfix {
        OriginPath {
            postfix: match_end.into(),
            origin: primer.origin,
        }
    }
    async fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<'a, 'g, T>,
    >(found: Self::Found, indexer: &'a mut Indexer<T, D>) -> Self::Indexed {
        OriginPath {
            origin: found.origin,
            postfix: BaseResult::index_found::<_, D>(found.postfix, indexer).await
        }
    }
}
#[async_trait]
pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection, Q: TraversalQuery, R: ResultKind>: Sized + Send + Sync {
    type Trav: Traversable<'a, 'g, T>;
    type Break: Send + Sync;
    type Continue: Send + Sync;

    async fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>
    ) -> ControlFlow<Self::Break, Self::Continue>;
}