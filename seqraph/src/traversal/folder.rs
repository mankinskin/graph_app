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