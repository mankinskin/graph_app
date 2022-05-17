use std::ops::ControlFlow;

use crate::{Tokenize, MatchDirection};

use super::*;

pub(crate) trait TraversalFolder<'a: 'g, 'g, T: Tokenize, D: MatchDirection, Q: TraversalQuery>: Sized {
    type Trav: Traversable<'a, 'g, T>;
    type Path: TraversalPath;
    type Node: ToTraversalNode<Q, Self::Path>;
    type Break;
    type Continue;
    fn fold_found(
        trav: &'a Self::Trav,
        acc: Self::Continue,
        node: Self::Node
    ) -> ControlFlow<Self::Break, Self::Continue>;
}