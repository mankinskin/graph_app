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
    type Advanced: Advanced + PathPop<Result=Self::Postfix>;//+ From<Self::Primer>;
    type Indexed: Send + Sync;
    //type Result: From<Self::Found>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<PathLeaf>) -> Self::Postfix;
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
    + HasRootedPath<Start>
    + GraphChild<Start>
    + PathAppend
    + From<PathLeaf>
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
    + HasRootedPath<Start>
    + GraphChild<Start>
    + PathAppend
    + PathAppend
    + From<PathLeaf>
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
    + PathSimplify
    + IntoRangePath
    + GetCacheKey
    + Send + Sync
{
    fn new_complete(child: Child, origin: ChildPath) -> Self;
    fn new_path(start: impl Into<ChildPath>, origin: ChildPath) -> Self;
}
impl<P: MatchEndPath + PathPop<Result=Self> + GetCacheKey + RootChild> Postfix for MatchEnd<P> {
    fn new_complete(c: Child, _origin: ChildPath) -> Self {
        Self::Complete(c)
    }
    fn new_path(start: impl Into<ChildPath>, _origin: ChildPath) -> Self {
        Self::Path(P::from(start.into()))
    }
}
impl<P: Postfix + RangePath + GetCacheKey + GraphRoot> Postfix for OriginPath<P> {
    fn new_complete(c: Child, origin: ChildPath) -> Self {
        Self {
            postfix: P::new_complete(c, origin.clone()),
            origin: MatchEnd::Path(origin),
        }
    }
    fn new_path(start: impl Into<ChildPath>, origin: ChildPath) -> Self {
        Self {
            postfix: P::new_path(start, origin.clone()),
            origin: MatchEnd::Path(origin),
        }
    }
}
pub trait Advanced:
    RootChild
    + NewAdvanced
    + HasRootedPath<Start>
    + HasRootedPath<End>
    + GraphChild<Start>
    + GraphChild<End>
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
    + HasRootedPath<Start>
    + HasRootedPath<End>
    + GraphChild<Start>
    + GraphChild<End>
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
    type Primer = ChildPath;
    type Postfix = MatchEnd<ChildPath>;
    type Advanced = SearchPath;
    type Indexed = Child;
    
    fn into_postfix(_primer: Self::Primer, match_end: MatchEnd<PathLeaf>) -> Self::Postfix {
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
    type Primer = OriginPath<ChildPath>;
    type Postfix = OriginPath<MatchEnd<ChildPath>>;
    type Advanced = OriginPath<SearchPath>;
    type Indexed = OriginPath<Child>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<PathLeaf>) -> Self::Postfix {
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