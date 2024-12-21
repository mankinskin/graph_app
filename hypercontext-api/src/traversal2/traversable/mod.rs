pub mod impls;

use std::{
    ops::{
        Deref,
        DerefMut,
    },
    sync::{
        RwLockReadGuard,
        RwLockWriteGuard,
    },
};

use crate::{
    graph::{
        kind::GraphKind,
        HypergraphRef,
        Hypergraph,
    },
    traversal::{
        context::TraversalStateContext,
        iterator::{
            IterKind,
            IterTrav,
            TraversalIterator,
        },
    },
};

pub trait Traversable: Sized + std::fmt::Debug {
    type Kind: GraphKind;
    type Guard<'a>: Traversable<Kind = Self::Kind> + Deref<Target = Hypergraph<Self::Kind>>
    where
        Self: 'a;
    fn graph(&self) -> Self::Guard<'_>;
}

pub type GraphKindOf<T> = <T as Traversable>::Kind;
pub(crate) type DirectionOf<T> = <GraphKindOf<T> as GraphKind>::Direction;

pub type TravKind<Trav> = <Trav as Traversable>::Kind;
pub type TravDir<Trav> = <TravKind<Trav> as GraphKind>::Direction;
pub type TravToken<Trav> = <TravKind<Trav> as GraphKind>::Token;
