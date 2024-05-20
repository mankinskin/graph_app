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
        kind::{
            BaseGraphKind,
            GraphKind,
        },
        HypergraphRef,
    },
    search::Searcher,
    traversal::{
        context::TraversalContext,
        iterator::{
            IterKind,
            IterTrav,
            TraversalIterator,
        },
    },
};

macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> Traversable for $target {
            type Kind = BaseGraphKind;
            type Guard<$lt> = $guard where Self: $lt, Self::Kind: $lt;
            fn graph<'a>(&'a $self_) -> Self::Guard<'a> {
                $func
            }
        }
    }
}
macro_rules! impl_traversable_mut {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $(, $par $(: $bhead $( + $btail )* )? ),* )?> TraversableMut for $target {
            type GuardMut<$lt> = $guard where Self: $lt;
            fn graph_mut<'a>(&'a mut $self_) -> Self::GuardMut<'a> {
                $func
            }
        }
    }
}
use crate::{
    graph::Hypergraph,
    index::indexer::Indexer,
};
pub(crate) use impl_traversable;
pub(crate) use impl_traversable_mut;

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

impl_traversable! {
    impl for &'_ Hypergraph, self => self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for &'_ mut Hypergraph, self => *self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for RwLockReadGuard<'_, Hypergraph>, self => &*self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for RwLockWriteGuard<'_, Hypergraph>, self => &**self; <'a> &'a Hypergraph
}
impl_traversable! {
    impl for HypergraphRef, self => self.read().unwrap(); <'a> RwLockReadGuard<'a, Hypergraph>
}
impl<T: Traversable> Traversable for Searcher<T> {
    type Kind = T::Kind;
    type Guard<'a> = T::Guard<'a> where T: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.graph.graph()
    }
}
impl<'g, T: Traversable> Traversable for &'g Searcher<T> {
    type Kind = T::Kind;
    type Guard<'a> = T::Guard<'a> where T: 'a, 'g: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.graph.graph()
    }
}
impl<'c, 'g, 'b: 'g, I: TraversalIterator<'b>> Traversable for &'c TraversalContext<'g, 'b, I> {
    type Kind = IterKind<'b, I>;
    type Guard<'a> = <IterTrav<'b, I> as Traversable>::Guard<'a> where I: 'a, 'c: 'a;
    fn graph(&self) -> Self::Guard<'_> {
        self.trav().graph()
    }
}
impl<'c, 'g, 'b: 'g, I: TraversalIterator<'b>> Traversable for &'c mut TraversalContext<'g, 'b, I> {
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

impl_traversable! {
    impl for Indexer,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ mut Indexer,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for Indexer,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for &'_ mut Indexer,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
//impl_traversable! {
//    impl <D: IndexDirection> for ReadContext<'p>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<T>>
//}
//impl_traversable! {
//    impl<D: IndexDirection> for &'_ ReadContext<'p>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<T>>
//}
//impl_traversable! {
//    impl<D: IndexDirection> for &'_ mut ReadContext<'p>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<T>>
//}

//impl_traversable! {
//    impl for ReadContext<'_>,
//    self => self.graph;
//    <'a> RwLockWriteGuard<'a, Hypergraph>
//}
//impl_traversable! {
//    impl for &'_ mut ReadContext<'_>,
//    self => self.graph;
//    <'a> RwLockWriteGuard<'a, Hypergraph>
//}
//impl_traversable_mut! {
//    impl for ReadContext<'_>,
//    self => self.graph;
//    <'a> RwLockWriteGuard<'a, Hypergraph>
//}
//impl_traversable_mut! {
//    impl for &'_ mut ReadContext<'_>,
//    self => self.graph;
//    <'a> RwLockWriteGuard<'a, Hypergraph>
//}
