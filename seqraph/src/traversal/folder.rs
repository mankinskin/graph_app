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
    = PathPair<FolderQuery<'a, 'g, T, D, Q, R, Ty>>;

pub(crate) trait ResultKind {
    type Result<P: MatchEndPath>: PostfixPath + From<P> + FromMatchEnd<P>;
}
pub(crate) struct MatchEndResult;
impl ResultKind for MatchEndResult {
    type Result<P: MatchEndPath> = MatchEnd<P>;
}
pub(crate) struct OriginPathResult;
impl ResultKind for OriginPathResult {
    type Result<P: MatchEndPath> = OriginPath<P>;
}
pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection, Q: TraversalQuery, R: ResultKind>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Break;
    type Continue;
    type AfterEndMatch: PostfixPath;
    fn fold_found(
        trav: &'a Self::Trav,
        acc: Self::Continue,
        node: TraversalNode<Self::AfterEndMatch, Q>
    ) -> ControlFlow<Self::Break, Self::Continue>;
}