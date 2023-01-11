use crate::*;
use super::*;

use std::ops::ControlFlow;

pub type Folder<T, D, Q, R, Ty>
    = <Ty as DirectedTraversalPolicy<T, D, Q, R>>::Folder;

pub trait FolderQ<
    T: Tokenize,
    D: MatchDirection,
    Q: BaseQuery,
    R: ResultKind,
> {
    type Query: BaseQuery;
}

impl<
    T: Tokenize,
    D: MatchDirection,
    Q: BaseQuery,
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
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<ChildPath<Start>>) -> Self::Postfix;
    fn index_found<
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<T>,
    >(found: Self::Found, indexer: &mut Indexer<T, D>) -> Self::Indexed;
}
pub trait Found<R: ResultKind>
    : RangePath
    + RoleChildPath
    + FromAdvanced<<R as ResultKind>::Advanced>
    + From<<R as ResultKind>::Postfix>
    + Wide
    + GetCacheKey
    + BasePath
    + GraphRoot
    + Ord
{
}
impl<
    R: ResultKind,
    T: RangePath
    + RoleChildPath
    + FromAdvanced<<R as ResultKind>::Advanced>
    + From<<R as ResultKind>::Postfix>
    + Wide
    + GetCacheKey
    + BasePath
    + GraphRoot
    + Ord
> Found<R> for T {
}
pub trait PathPrimer<R: ResultKind>:
    RoleChildPath
    + NodePath<Start>
    + HasRolePath<Start>
    + GraphRootChild<Start>
    //+ PathAppend
    + From<ChildPath<Start>>
    + Into<R::Postfix>
    + IntoAdvanced<R>
    + Wide
    + GetCacheKey
    + Send
    + Sync
    + Unpin
{
}
impl<
    R: ResultKind,
    T: NodePath<Start>
    + RoleChildPath
    + HasRolePath<Start>
    + GraphRootChild<Start>
    //+ PathAppend
    + From<ChildPath<Start>>
    + IntoAdvanced<R>
    + Into<R::Postfix>
    + Wide
    + GetCacheKey
    + BasePath
> PathPrimer<R> for T
{
}

pub trait RoleChildPath {
    fn role_path_child_location<
        R: PathRole,
    >(&self) -> ChildLocation
        where Self: PathChild<R>
    {
        PathChild::<R>::path_child_location(self)
    }
    fn role_child_location<
        R: PathRole,
    >(&self) -> ChildLocation
        where Self: HasRolePath<R>
    {
        self.child_path::<R>().child_location()
    }
    fn child_path<
        R: PathRole,
    >(&self) -> &ChildPath<R>
        where Self: HasRolePath<R>
    {
        HasRolePath::<R>::role_path(self)
    }
    fn child_path_mut<
        R: PathRole,
    >(&mut self) -> &mut ChildPath<R>
        where Self: HasRolePath<R>
    {
        HasRolePath::<R>::role_path_mut(self)
    }
}
impl<T> RoleChildPath for T {
}
pub trait Postfix:
    NodePath<Start>
    + PathSimplify
    //+ IntoRangePath
    + GetCacheKey
    + BasePath
    + GraphRoot
{
    fn new_complete(child: Child, origin: ChildPath<Start>) -> Self;
    fn new_path(start: impl Into<ChildPath<Start>>, origin: ChildPath<Start>) -> Self;
}
impl<P: MatchEndPath + PathPop<Result=Self> + GetCacheKey + NodePath<Start> + GraphRoot> Postfix for MatchEnd<P> {
    fn new_complete(c: Child, _origin: ChildPath<Start>) -> Self {
        Self::Complete(c)
    }
    fn new_path(start: impl Into<ChildPath<Start>>, _origin: ChildPath<Start>) -> Self {
        Self::Path(P::from(start.into()))
    }
}
impl<P: Postfix> Postfix for OriginPath<P> {
    fn new_complete(c: Child, origin: ChildPath<Start>) -> Self {
        Self {
            postfix: P::new_complete(c, origin.clone()),
            origin: MatchEnd::Path(origin),
        }
    }
    fn new_path(start: impl Into<ChildPath<Start>>, origin: ChildPath<Start>) -> Self {
        Self {
            postfix: P::new_path(start, origin.clone()),
            origin: MatchEnd::Path(origin),
        }
    }
}
pub trait Advanced:
    RoleChildPath
    + NodePath<Start>
    + BasePath
    + HasRolePath<Start>
    + HasRolePath<End>
    + GraphRootChild<Start>
    + GraphRootChild<End>
    + PathComplete
    + AddMatchWidth
    + GetCacheKey
    + PathChild<Start>
    + PathChild<End>
    + AdvanceExit
    + GraphRoot
{
    fn role_path_child<
        R: PathRole,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&self, trav: &Trav) -> Child
        where Self: PathChild<R>
    {
        PathChild::<R>::path_child(self, trav)
    }
    fn child_pos<
        R: PathRole,
    >(&self) -> usize
        where Self: HasRolePath<R>
    {
        HasRolePath::<R>::role_path(self).root_child_pos()
    }
    fn raw_child_path<
        R: PathRole,
    >(&self) -> &Vec<ChildLocation>
        where Self: HasRolePath<R>
    {
        HasRolePath::<R>::role_path(self).path()
    }
    fn raw_child_path_mut<
        R: PathRole,
    >(&mut self) -> &mut Vec<ChildLocation>
        where Self: HasRolePath<R>
    {
        HasRolePath::<R>::role_path_mut(self).path_mut()
    }
}
impl<
    T:
    RoleChildPath
    +NodePath<Start>
    + BasePath
    + HasRolePath<Start>
    + HasRolePath<End>
    + GraphRootChild<Start>
    + GraphRootChild<End>
    + PathComplete
    + AddMatchWidth
    + GetCacheKey
    + PathChild<Start>
    + PathChild<End>
    + AdvanceExit
> Advanced for T {
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BaseResult;


impl ResultKind for BaseResult {
    type Found = FoundPath;
    type Primer = ChildPath<Start>;
    type Advanced = SearchPath;
    type Postfix = MatchEnd<ChildPath<Start>>;
    type Indexed = Child;
    
    fn into_postfix(_primer: Self::Primer, match_end: MatchEnd<ChildPath<Start>>) -> Self::Postfix {
        match_end.into()
    }
    fn index_found<
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<T>,
    >(found: Self::Found, indexer: &mut Indexer<T, D>) -> Self::Indexed {
        indexer.index_found(found.into_range_path().into())
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct OriginPathResult;

impl ResultKind for OriginPathResult {
    type Found = OriginPath<FoundPath>;
    type Primer = OriginPath<ChildPath<Start>>;
    type Postfix = OriginPath<MatchEnd<ChildPath<Start>>>;
    type Advanced = OriginPath<SearchPath>;
    type Indexed = OriginPath<Child>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<ChildPath<Start>>) -> Self::Postfix {
        OriginPath {
            postfix: match_end.into(),
            origin: primer.origin,
        }
    }
    fn index_found<
        T: Tokenize,
        D: IndexDirection,
        //Trav: TraversableMut<T>,
    >(found: Self::Found, indexer: &mut Indexer<T, D>) -> Self::Indexed {
        OriginPath {
            origin: found.origin,
            postfix: BaseResult::index_found::<_, D>(found.postfix, indexer)
        }
    }
}

pub trait TraversalFolder<T: Tokenize, D: MatchDirection, Q: BaseQuery, R: ResultKind>: Sized + Send + Sync {
    type Trav: Traversable<T>;
    type Break: Send + Sync;
    type Continue: Send + Sync;

    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>
    ) -> ControlFlow<Self::Break, Self::Continue>;
}