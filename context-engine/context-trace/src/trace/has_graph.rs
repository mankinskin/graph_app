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

use crate::graph::{
    Hypergraph,
    HypergraphRef,
    kind::GraphKind,
};

pub trait HasGraph: Sized + std::fmt::Debug {
    type Kind: GraphKind;
    type Guard<'a>: HasGraph<Kind = Self::Kind>
        + Deref<Target = Hypergraph<Self::Kind>>
    where
        Self: 'a;
    fn graph(&self) -> Self::Guard<'_>;
}
impl<T: HasGraph> HasGraph for &T {
    type Kind = TravKind<T>;
    type Guard<'g>
        = <T as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        (**self).graph()
    }
}
impl<T: HasGraph> HasGraph for &mut T {
    type Kind = TravKind<T>;
    type Guard<'g>
        = <T as HasGraph>::Guard<'g>
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        (**self).graph()
    }
}
pub type TravKind<G> = <G as HasGraph>::Kind;
pub type TravDir<G> = <TravKind<G> as GraphKind>::Direction;
pub type TravToken<G> = <TravKind<G> as GraphKind>::Token;

#[macro_export]
macro_rules! impl_has_graph {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> $crate::trace::has_graph::HasGraph for $target {
            type Kind = $crate::graph::kind::BaseGraphKind;
            type Guard<$lt> = $guard where Self: $lt, Self::Kind: $lt;
            fn graph(&$self_) -> Self::Guard<'_> {
                $func
            }
        }
    }
}
#[macro_export]
macro_rules! impl_has_graph_mut {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $(, $par $(: $bhead $( + $btail )* )? ),* )?> $crate::trace::has_graph::HasGraphMut for $target {
            type GuardMut<$lt> = $guard where Self: $lt;
            fn graph_mut(&mut $self_) -> Self::GuardMut<'_> {
                $func
            }
        }
    }
}
pub use impl_has_graph;
pub use impl_has_graph_mut;

impl_has_graph! {
    impl for RwLockReadGuard<'_, Hypergraph>,
    self => self; <'a> &'a Hypergraph
}
impl_has_graph! {
    impl for RwLockWriteGuard<'_, Hypergraph>,
    self => &**self; <'a> &'a Hypergraph
}
impl_has_graph! {
    impl for HypergraphRef, self => self.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}

//impl_has_graph! {
//    impl<G: GraphKind> for Hypergraph<G>, self => self; <'g> &'g Self
//}

impl<G: GraphKind> HasGraph for Hypergraph<G> {
    type Kind = G;
    type Guard<'g>
        = &'g Self
    where
        Self: 'g;
    fn graph(&self) -> Self::Guard<'_> {
        self
    }
}
pub trait HasGraphMut: HasGraph {
    type GuardMut<'a>: HasGraphMut<Kind = Self::Kind>
        + Deref<Target = Hypergraph<Self::Kind>>
        + DerefMut
    where
        Self: 'a;
    fn graph_mut(&mut self) -> Self::GuardMut<'_>;
}

impl<T: HasGraphMut> HasGraphMut for &mut T {
    type GuardMut<'g>
        = <T as HasGraphMut>::GuardMut<'g>
    where
        Self: 'g;
    fn graph_mut(&mut self) -> Self::GuardMut<'_> {
        (**self).graph_mut()
    }
}

impl_has_graph_mut! {
    impl for Hypergraph, self => self; <'a> &'a mut Self
}
impl_has_graph_mut! {
    impl for RwLockWriteGuard<'_, Hypergraph>, self => &mut **self; <'a> &'a mut Hypergraph
}
impl_has_graph_mut! {
    impl for HypergraphRef, self => self.write().unwrap(); <'a> RwLockWriteGuard<'a, Hypergraph>
}
