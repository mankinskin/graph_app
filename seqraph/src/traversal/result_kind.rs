use crate::*;
use super::*;

//pub trait ResultKind: Eq + Clone + Debug + Send + Sync + Unpin {
//    type Query: QueryPath;
//    type Found: Found<Self> + From<FoundPath>;
//    type Primer: PathPrimer<Self>;
//    type Postfix: Postfix + From<MatchEnd<RootedRolePath<Start>>> + From<Self::Primer> + IntoPrimer<Self>;
//    type Advanced: Advanced + PathPop + Into<Self::Postfix> + Into<RootedRolePath<End>>;
//    type Indexed: Send + Sync;
//    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<RootedRolePath<Start>>) -> Self::Postfix;
//}
pub type Primer = RootedRolePath<Start>;
pub type Postfix = MatchEnd<Primer>;

pub trait Found
    : BasePath
    + PathComplete
    //+ RoleChildPath
    //+ FromAdvanced<<R as ResultKind>::Advanced>
    //+ From<<R as ResultKind>::Postfix>
    //+ Wide
    //+ GetCacheKey
    //+ GraphRoot
    //+ Ord
{
}
impl<
    T: BasePath
    + PathComplete
    //+ RoleChildPath
    //+ FromAdvanced<<R as ResultKind>::Advanced>
    //+ From<<R as ResultKind>::Postfix>
    //+ Wide
    //+ GetCacheKey
    //+ GraphRoot
    //+ Ord
> Found for T {
}
pub trait PathPrimer:
    RoleChildPath
    + NodePath<Start>
    //+ HasRolePath<Start>
    + GraphRootChild<Start>
    + From<RootedRolePath<Start>>
    + From<SearchPath>
    + Into<Postfix>
    + IntoAdvanced
    //+ Wide
    + RootKey
    + Send
    + Sync
    + Unpin
    + Hash
    + HasSinglePath
    + MatchEndPath
{
}
impl<
    T: NodePath<Start>
    + RoleChildPath
    //+ HasRolePath<Start>
    + GraphRootChild<Start>
    + From<RootedRolePath<Start>>
    + From<SearchPath>
    + IntoAdvanced
    + Into<Postfix>
    //+ Wide
    + RootKey
    + BasePath
    + Hash
    + HasSinglePath
    + MatchEndPath
> PathPrimer for T
{
}

pub trait RoleChildPath {
    fn role_leaf_child_location<
        R: PathRole,
    >(&self) -> Option<ChildLocation>
        where Self: LeafChild<R>
    {
        LeafChild::<R>::leaf_child_location(self)
    }
    fn role_root_child_location<
        R: PathRole,
    >(&self) -> ChildLocation
        where Self: GraphRootChild<R>
    {
        GraphRootChild::<R>::root_child_location(self)
    }
    fn child_path_mut<
        R: PathRole,
    >(&mut self) -> &mut RolePath<R>
        where Self: HasRolePath<R>
    {
        HasRolePath::<R>::role_path_mut(self)
    }
    fn role_leaf_child<
        R: PathRole,
        Trav: Traversable,
    >(&self, trav: &Trav) -> Child
        where Self: LeafChild<R>
    {
        LeafChild::<R>::leaf_child(self, trav)
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
impl<T> RoleChildPath for T {
}
//pub trait Postfix:
//    NodePath<Start>
//    + PathSimplify
//    //+ IntoRangePath
//    + RootKey
//    + BasePath
//    + GraphRoot
//    + Into<MatchEnd<RootedRolePath<Start>>>
//{
//    //fn new_complete(child: Child, origin: RootedRolePath<Start>) -> Self;
//    //fn new_path(start: impl Into<RootedRolePath<Start>>, origin: RootedRolePath<Start>) -> Self;
//}
//impl<T:
//    NodePath<Start>
//    + PathSimplify
//    + RootKey
//    + BasePath
//    + GraphRoot
//    + Into<MatchEnd<RootedRolePath<Start>>>
//> Postfix for T {
//}
//impl<P: MatchEndPath + Postfix> Postfix for MatchEnd<P> {
//    //fn new_complete(c: Child, _origin: RootedRolePath<Start>) -> Self {
//    //    Self::Complete(c)
//    //}
//    //fn new_path(start: impl Into<RootedRolePath<Start>>, _origin: RootedRolePath<Start>) -> Self {
//    //    Self::Path(P::from(start.into()))
//    //}
//}
//impl<P: Postfix> Postfix for OriginPath<P> {
//    fn new_complete(c: Child, origin: RolePath<Start>) -> Self {
//        Self {
//            postfix: P::new_complete(c, origin.clone()),
//            origin: MatchEnd::Path(origin),
//        }
//    }
//    fn new_path(start: impl Into<RolePath<Start>>, origin: RolePath<Start>) -> Self {
//        Self {
//            postfix: P::new_path(start, origin.clone()),
//            origin: MatchEnd::Path(origin),
//        }
//    }
//}
pub trait Advanced:
    RoleChildPath
    + NodePath<Start>
    + BasePath
    + HasRolePath<Start>
    + HasRolePath<End>
    + GraphRootChild<Start>
    + GraphRootChild<End>
    + GetCacheKey
    + LeafChild<Start>
    + LeafChild<End>
    + AdvanceRootPos<End>
    + RootChildPosMut<End>
    + GraphRoot
    + PathAppend
{
}
impl<
    T:
    RoleChildPath
    + NodePath<Start>
    + BasePath
    + HasRolePath<Start>
    + HasRolePath<End>
    + GetCacheKey
    + GraphRootChild<Start>
    + GraphRootChild<End>
    + LeafChild<Start>
    + LeafChild<End>
    + AdvanceRootPos<End>
    + RootChildPosMut<End>
    + PathAppend
> Advanced for T {
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BaseResult;


//impl ResultKind for BaseResult {
//    type Query = QueryRangePath;
//    type Found = FoundPath;
//    type Primer = RootedRolePath<Start>;
//    type Advanced = SearchPath;
//    type Postfix = MatchEnd<RootedRolePath<Start>>;
//    type Indexed = Child;
//    
//    fn into_postfix(_primer: Self::Primer, match_end: MatchEnd<RootedRolePath<Start>>) -> Self::Postfix {
//        match_end.into()
//    }
//    //fn index_found<
//    //    T: Tokenize,
//    //    D: IndexDirection,
//    //    //Trav: TraversableMut<T>,
//    //>(found: Self::Found, indexer: &mut Indexer<T, D>) -> Self::Indexed {
//    //    indexer.index_found(found.into_range_path().into())
//    //}
//}

//#[derive(Eq, PartialEq, Clone, Debug)]
//pub struct OriginPathResult;
//
//impl ResultKind for OriginPathResult {
//    type Found = OriginPath<FoundPath<Self, QueryRangePath>>;
//    type Primer = OriginPath<RolePath<Start>>;
//    type Postfix = OriginPath<MatchEnd<RolePath<Start>>>;
//    type Advanced = OriginPath<SearchPath>;
//    type Indexed = OriginPath<Child>;
//    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<RolePath<Start>>) -> Self::Postfix {
//        OriginPath {
//            postfix: match_end.into(),
//            origin: primer.origin,
//        }
//    }
//    //fn index_found<
//    //    T: Tokenize,
//    //    D: IndexDirection,
//    //    //Trav: TraversableMut<T>,
//    //>(found: Self::Found, indexer: &mut Indexer<T, D>) -> Self::Indexed {
//    //    OriginPath {
//    //        origin: found.origin,
//    //        postfix: BaseResult::index_found::<_, D>(found.postfix, indexer)
//    //    }
//    //}
//}
//