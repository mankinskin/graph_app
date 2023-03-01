use crate::*;

macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <$( $(, $par $(: $bhead $( + $btail )* )? ),* )?> Traversable for $target {
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
pub(crate) use impl_traversable;
pub(crate) use impl_traversable_mut;

pub trait Traversable: Sized + std::fmt::Debug {
    type Kind: GraphKind;
    type Guard<'a>: Traversable<Kind=Self::Kind> + Deref<Target=Hypergraph<Self::Kind>> where Self: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a>;
}
pub type TravDir<Trav> = <<Trav as Traversable>::Kind as GraphKind>::Direction;
pub type TravToken<Trav> = <<Trav as Traversable>::Kind as GraphKind>::Token;

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
impl_traversable! {
    impl for Searcher,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}
impl_traversable! {
    impl for &'_ Searcher,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph>
}

impl_traversable! {
    impl for Hypergraph, self => self; <'a> &'a Self
}
pub trait TraversableMut: Traversable {
    type GuardMut<'a>:
        TraversableMut<Kind=Self::Kind>
        + Deref<Target=Hypergraph<Self::Kind>>
        + DerefMut where Self: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a>;
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
impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Traversable for Pather<Side> {
    type Kind = BaseGraphKind;
    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a> {
        self.indexer.graph()
    }
}
impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> TraversableMut for Pather<Side> {
    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
        self.indexer.graph_mut()
    }
}

impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Traversable for Splitter<Side> {
    type Kind = BaseGraphKind;
    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a> {
        self.indexer.graph()
    }
}
impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> TraversableMut for Splitter<Side> {
    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
        self.indexer.graph_mut()
    }
}

impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> Traversable for Contexter<Side> {
    type Kind = BaseGraphKind;
    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a> {
        self.indexer.graph()
    }
}

impl<Side: IndexSide<<BaseGraphKind as GraphKind>::Direction>> TraversableMut for Contexter<Side> {
    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
        self.indexer.graph_mut()
    }
}

//impl_traversable! {
//    impl <D: IndexDirection> for Reader<T, D>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<T>>
//}
//impl_traversable! {
//    impl<D: IndexDirection> for &'_ Reader<T, D>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<T>>
//}
//impl_traversable! {
//    impl<D: IndexDirection> for &'_ mut Reader<T, D>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<T>>
//}

//impl_traversable_mut! {
//    impl<D: IndexDirection> for Reader<T, D>,
//    self => self.graph.write().unwrap();
//    <'a> RwLockWriteGuard<'a, Hypergraph<T>>
//}
//impl_traversable_mut! {
//    impl<D: IndexDirection> for &'_ mut Reader<T, D>,
//    self => self.graph.write().unwrap();
//    <'a> RwLockWriteGuard<'a, Hypergraph<T>>
//}
