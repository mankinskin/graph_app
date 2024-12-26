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

pub trait Traversable: Sized + std::fmt::Debug {
    type Kind: GraphKind;
    type Guard<'a>: Traversable<Kind = Self::Kind> + Deref<Target = Hypergraph<Self::Kind>>
    where
        Self: 'a;
    fn graph(&self) -> Self::Guard<'_>;
}
impl<'a, T: Traversable> Traversable for &'a T {
    type Kind = TravKind<T>;
    type Guard<'g> = <T as Traversable>::Guard<'g> where Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        (**self).graph()
    }
}
impl<'a, T: Traversable> Traversable for &'a mut T {
    type Kind = TravKind<T>;
    type Guard<'g> = <T as Traversable>::Guard<'g> where Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        (**self).graph()
    }
}
pub type GraphKindOf<T> = <T as Traversable>::Kind;
pub(crate) type DirectionOf<T> = <GraphKindOf<T> as GraphKind>::Direction;

pub type TravKind<Trav> = <Trav as Traversable>::Kind;
pub type TravDir<Trav> = <TravKind<Trav> as GraphKind>::Direction;
pub type TravToken<Trav> = <TravKind<Trav> as GraphKind>::Token;

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

use crate::graph::{kind::GraphKind, Hypergraph, HypergraphRef};

use super::fold::{TraversalContext, TraversalKind};


//impl_traversable! {
//    impl for &'_ Hypergraph,
//    self => self; <'a> &'a Hypergraph
//}
//impl_traversable! {
//    impl for &'_ mut Hypergraph,
//    self => *self; <'a> &'a Hypergraph
//}
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


impl<'a, K: TraversalKind> Traversable for &'a TraversalContext<'a, K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g> = <K::Trav as Traversable>::Guard<'g> where Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}

impl<'a, K: TraversalKind> Traversable for &'a mut TraversalContext<'a, K> {
    type Kind = TravKind<K::Trav>;
    type Guard<'g> = <K::Trav as Traversable>::Guard<'g> where Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav.graph()
    }
}
//impl<'c, 'g, 'b: 'g, K: TraversalKind> Traversable for &'c TraversalStateContext<'g, 'b, K> {
//    type Kind = TravKind<K::Trav>;
//    type Guard<'a> = <K::Trav as Traversable>::Guard<'a> where K::Trav: 'a, 'c: 'a;
//    fn graph(&self) -> Self::Guard<'_> {
//        self.ctx.trav.graph()
//    }
//}
//
//impl<'c, 'g, 'b: 'g, K: TraversalKind> Traversable for &'c mut TraversalStateContext<'g, 'b, K> {
//    type Kind = TravKind<K::Trav>;
//    type Guard<'a> = <K::Trav as Traversable>::Guard<'a> where K::Trav: 'a, 'c: 'a;
//    fn graph(&self) -> Self::Guard<'_> {
//        self.ctx.trav.graph()
//    }
//}

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
impl<'a, T: TraversableMut> TraversableMut for &'a mut T {
    type GuardMut<'g> = <T as TraversableMut>::GuardMut<'g> where Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_> {
        (**self).graph_mut()
    }
}

impl_traversable_mut! {
    impl for Hypergraph, self => self; <'a> &'a mut Self
}
//impl_traversable_mut! {
//    impl for &'_ mut Hypergraph, self => *self; <'a> &'a mut Hypergraph
//}
impl_traversable_mut! {
    impl for RwLockWriteGuard<'_, Hypergraph>, self => &mut **self; <'a> &'a mut Hypergraph
}
impl_traversable_mut! {
    impl for HypergraphRef, self => self.write().unwrap(); <'a> RwLockWriteGuard<'a, Hypergraph>
}


