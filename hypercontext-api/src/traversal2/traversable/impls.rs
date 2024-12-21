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

#[macro_export]
macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> Traversable for $target {
            type Kind = $crate::graph::kind::BaseGraphKind;
            type Guard<$lt> = $guard where Self: $lt, Self::Kind: $lt;
            fn graph(&$self_) -> Self::Guard<'_> {
                $func
            }
        }
    }
}
#[macro_export]
macro_rules! impl_traversable_mut {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $(, $par $(: $bhead $( + $btail )* )? ),* )?> TraversableMut for $target {
            type GuardMut<$lt> = $guard where Self: $lt;
            fn graph_mut(&mut $self_) -> Self::GuardMut<'_> {
                $func
            }
        }
    }
}
pub use impl_traversable;
pub use impl_traversable_mut;

impl_traversable! {
    impl for &'_ Hypergraph,
    self => self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for &'_ mut Hypergraph,
    self => *self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for RwLockReadGuard<'_, Hypergraph>,
    self => self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for RwLockWriteGuard<'_, Hypergraph>,
    self => &**self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for HypergraphRef, self => self.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}


impl<'c, 'g, 'b: 'g, I: TraversalIterator<'b>> Traversable for &'c TraversalStateContext<'g, 'b, I> {
    type Kind = IterKind<'b, I>;
    type Guard<'a> = <IterTrav<'b, I> as Traversable>::Guard<'a> where I: 'a, 'c: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav().graph()
    }
}

impl<'c, 'g, 'b: 'g, I: TraversalIterator<'b>> Traversable for &'c mut TraversalStateContext<'g, 'b, I> {
    type Kind = IterKind<'b, I>;
    type Guard<'a> = <IterTrav<'b, I> as Traversable>::Guard<'a> where I: 'a, 'c: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav().graph()
    }
}

impl_traversable! {
    impl for Hypergraph, self => self; <'a> &'a Self
}
pub trait TraversableMut: Traversable {
    type GuardMut<'a>: TraversableMut<Kind = Self::Kind>
        + Deref<Target = Hypergraph<Self::Kind>>
        + DerefMut
    where
        Self: 'a;
    fn graph_mut(&mut self) -> Self::GuardMut<'_>;
}

impl_traversable_mut! {
    impl for Hypergraph, self => self; <'a> &'a mut Self
}
impl_traversable_mut! {
    impl for &'_ mut Hypergraph, self => *self; <'a> &'a mut Hypergraph
}
impl_traversable_mut! {
    impl for RwLockWriteGuard<'_, Hypergraph>, self => &mut **self; <'a> &'a mut Hypergraph
}
impl_traversable_mut! {
    impl for HypergraphRef, self => self.write().unwrap(); <'a> RwLockWriteGuard<'a, Hypergraph>
}