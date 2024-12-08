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
        }, HypergraphRef,
        Hypergraph,
    },
    search::searcher::Searcher,
    traversal::{
        context::TraversalContext,
        iterator::{
            IterKind,
            IterTrav,
            TraversalIterator,
        },
    },
    insert::context::InsertContext,
    read::reader::context::ReadContext,
};

macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> Traversable for $target {
            type Kind = BaseGraphKind;
            type Guard<$lt> = $guard where Self: $lt, Self::Kind: $lt;
            fn graph(&$self_) -> Self::Guard<'_> {
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
            fn graph_mut(&mut $self_) -> Self::GuardMut<'_> {
                $func
            }
        }
    }
}
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
    impl for InsertContext,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ mut InsertContext,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for InsertContext,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for &'_ mut InsertContext,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}

impl_traversable! {
    impl for ReadContext<'_>,
    //Self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ ReadContext<'_>,
    //self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ mut ReadContext<'_>,
    //self => self.graph.graph();
    //<'a> &'a Hypergraph
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for ReadContext<'_>,
    //self => self.graph.graph_mut();
    //<'a> &'a mut Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
impl_traversable_mut! {
    impl for &'_ mut ReadContext<'_>,
    //self => self.graph.graph_mut();
    //<'a> &'a mut Hypergraph
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph>
}
