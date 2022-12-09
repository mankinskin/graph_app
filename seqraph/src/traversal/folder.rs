use crate::*;
use super::*;

use std::ops::ControlFlow;

pub type Folder<T, D, Q, R, Ty>
    = <Ty as DirectedTraversalPolicy<T, D, Q, R>>::Folder;

pub trait FolderQ<
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
> {
    type Query: TraversalQuery;
}

impl<
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    R: ResultKind,
    Ty: TraversalFolder<T, D, Q, R>,
> FolderQ<T, D, Q, R> for Ty {
    type Query = Q;
}

pub type FolderQuery<T, D, Q, R, Ty>
    = <Folder<T, D, Q, R, Ty> as FolderQ<T, D, Q, R>>::Query;

pub type FolderPathPair<T, D, Q, R, Ty>
    = PathPair<<R as ResultKind>::Advanced, FolderQuery<T, D, Q, R, Ty>>;


pub trait ResultKind: Eq + Clone + Debug + Send + Sync + Unpin {
    type Found: Found<Self>;
    type Primer: PathPrimer<Self>;
    type Postfix: Postfix + PathAppend<Result=Self::Primer> + From<Self::Primer>;
    type Advanced: Advanced + PathPop<Result=Self::Postfix> + From<Self::Primer>;
    type Indexed: Send + Sync;
    //type Result: From<Self::Found>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<StartLeaf>) -> Self::Postfix;
    fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<T>,
    >(found: Self::Found, indexer: &'a mut Indexer<T, D>) -> Self::Indexed;
}
pub trait Found<R: ResultKind>
    : RangePath
    + FromAdvanced<<R as ResultKind>::Advanced>
    + From<<R as ResultKind>::Postfix>
    + Wide
    + GetCacheKey
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
    + GetCacheKey
    + Ord
    + Send
    + Sync
    + Unpin
> Found<R> for T {
}
pub trait PathPrimer<R: ResultKind>:
    NodePath
    + HasStartMatchPath
    + GraphEntry
    + PathAppend
    + From<StartLeaf>
    + From<R::Advanced>
    + Wide
    + GetCacheKey
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
    + GetCacheKey
    + Send
    + Sync
    + Unpin
> PathPrimer<R> for T
{
}

pub trait Postfix:
    NodePath
    + PathReduce
    + IntoRangePath
    + GetCacheKey
    + Send + Sync
{
    fn new_complete(child: Child, origin: StartPath) -> Self;
    fn new_path(start: impl Into<StartPath>, origin: StartPath) -> Self;
}
impl<P: MatchEndPath + PathPop<Result=Self> + GetCacheKey> Postfix for MatchEnd<P> {
    fn new_complete(c: Child, _origin: StartPath) -> Self {
        Self::Complete(c)
    }
    fn new_path(start: impl Into<StartPath>, _origin: StartPath) -> Self {
        Self::Path(P::from(start.into()))
    }
}
impl<P: Postfix + RangePath + GetCacheKey> Postfix for OriginPath<P> {
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
pub trait Advanced:
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
    + GetCacheKey
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
    + GetCacheKey
> Advanced for T {
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BaseResult;


impl ResultKind for BaseResult {
    type Found = FoundPath;
    type Primer = StartPath;
    type Postfix = MatchEnd<StartPath>;
    type Advanced = SearchPath;
    type Indexed = Child;
    fn into_postfix(_primer: Self::Primer, match_end: MatchEnd<StartLeaf>) -> Self::Postfix {
        match_end.into()
    }
    fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<T>,
    >(found: Self::Found, indexer: &'a mut Indexer<T, D>) -> Self::Indexed {
        indexer.index_found(found.into_range_path().into())
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct OriginPathResult;


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
    fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<T>,
    >(found: Self::Found, indexer: &'a mut Indexer<T, D>) -> Self::Indexed {
        OriginPath {
            origin: found.origin,
            postfix: BaseResult::index_found::<_, D>(found.postfix, indexer)
        }
    }
}

pub trait TraversalFolder<T: Tokenize, D: MatchDirection, Q: TraversalQuery, R: ResultKind>: Sized + Send + Sync {
    type Trav: Traversable<T>;
    type Break: Send + Sync;
    type Continue: Send + Sync;

    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>
    ) -> ControlFlow<Self::Break, Self::Continue>;
}