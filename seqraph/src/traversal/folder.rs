use std::ops::ControlFlow;

use crate::{Tokenize, MatchDirection};

use super::*;

pub(crate) type Folder<'a, 'g, T, D, Q, Ty>
    = <Ty as DirectedTraversalPolicy<'a, 'g, T, D, Q>>::Folder;

pub(crate) type FolderNode<'a, 'g, T, D, Q, Ty>
    = <Folder<'a, 'g, T, D, Q, Ty> as TraversalFolder<'a, 'g, T, D, Q>>::Node;

pub(crate) trait FolderQ<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
> {
    type Query: TraversalQuery;
}

impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: MatchDirection,
    Q: TraversalQuery,
    Ty: TraversalFolder<'a, 'g, T, D, Q>,
> FolderQ<'a, 'g, T, D, Q> for Ty {
    type Query = Q;
}

pub(crate) type FolderQuery<'a, 'g, T, D, Q, Ty>
    = <Folder<'a, 'g, T, D, Q, Ty> as FolderQ<'a, 'g, T, D, Q>>::Query;

#[allow(unused)]
pub(crate) type FolderPath<'a, 'g, T, D, Q, Ty>
    = <Folder<'a, 'g, T, D, Q, Ty> as TraversalFolder<'a, 'g, T, D, Q>>::Path;

pub(crate) type FolderStartPath<'a, 'g, T, D, Q, Ty>
    = <Folder<'a, 'g, T, D, Q, Ty> as TraversalFolder<'a, 'g, T, D, Q>>::StartPath;

pub(crate) type FolderPathPair<'a, 'g, T, D, Q, Ty>
    = PathPair<FolderQuery<'a, 'g, T, D, Q, Ty>, SearchPath>;

pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection, Q: TraversalQuery>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Path: TraversalPath;
    type StartPath: TraversalStartPath + From<Self::Path> + Into<StartPath>;
    type Node: ToTraversalNode<Q>;
    type Break;
    type Continue;
    fn fold_found(
        trav: &'a Self::Trav,
        acc: Self::Continue,
        node: Self::Node
    ) -> ControlFlow<Self::Break, Self::Continue>;
}