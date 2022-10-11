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

pub(crate) trait ResultKind: Eq + Clone + Debug {
    type Found: Found<Self>;
    type Primer: PathPrimer<Self>;
    type Postfix: Postfix + PathAppend<Result=Self::Primer> + From<Self::Primer>;
    type Advanced: Advanced + PathPop<Result=Self::Postfix> + From<Self::Primer>;
    type Indexed;
    //type Result: From<Self::Found>;
    fn into_postfix(primer: Self::Primer, match_end: MatchEnd<StartLeaf>) -> Self::Postfix;
    fn index_found<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        Trav: TraversableMut<'a, 'g, T>,
    >(found: Self::Found, trav: &'a mut Trav) -> Self::Indexed;
}
pub(crate) trait Found<R: ResultKind>: RangePath + FromAdvanced<<R as ResultKind>::Advanced> + From<<R as ResultKind>::Postfix> + Wide + Ord {
}
impl<R: ResultKind, T: RangePath + FromAdvanced<<R as ResultKind>::Advanced> + From<<R as ResultKind>::Postfix> + Wide + Ord> Found<R> for T {
}
pub(crate) trait PathPrimer<R: ResultKind>:
    NodePath
    + GraphEntry
    + PathAppend
    + From<StartLeaf>
    + From<R::Advanced>
    + Wide
{
}
impl<
    R: ResultKind,
    T: NodePath
    + PathAppend
    + GraphEntry
    + PathAppend
    + From<StartLeaf>
    + From<<R as ResultKind>::Advanced>
    + Wide
> PathPrimer<R> for T
{
}

pub(crate) trait Postfix: NodePath + PathReduce + IntoRangePath {
    fn new_complete(child: Child, origin: StartPath) -> Self;
    fn new_path(start: impl Into<StartPath>, origin: StartPath) -> Self;
}
impl<P: MatchEndPath> Postfix for MatchEnd<P> {
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
> Advanced for T {
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct BaseResult;

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
        Trav: TraversableMut<'a, 'g, T>,
    >(found: Self::Found, trav: &'a mut Trav) -> Self::Indexed {
        Indexing::<_, D>::index_found(trav, found.into_range_path().into())
    }
}
#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct OriginPathResult;
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
        Trav: TraversableMut<'a, 'g, T>,
    >(found: Self::Found, trav: &'a mut Trav) -> Self::Indexed {
        OriginPath {
            origin: found.origin,
            postfix: BaseResult::index_found::<_, D, _>(found.postfix, trav)
        }
    }
}
pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection, Q: TraversalQuery, R: ResultKind>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Break;
    type Continue;
    //type Primer: PathPrimer;
    fn fold_found(
        trav: &'a Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<R, Q>
    ) -> ControlFlow<Self::Break, Self::Continue>;
}