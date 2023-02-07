use crate::*;

macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <G: GraphKind $( $(, $par $(: $bhead $( + $btail )* )? ),* )?> Traversable for $target {
            type Kind = G;
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
        impl <G: GraphKind $( $(, $par $(: $bhead $( + $btail )* )? ),* )?> TraversableMut for $target {
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
    impl for &'_ Hypergraph<G>, self => self; <'a> &'a Hypergraph<G>
}
impl_traversable! {
    impl for &'_ mut Hypergraph<G>, self => *self; <'a> &'a Hypergraph<G>
}
impl_traversable! {
    impl for RwLockReadGuard<'_, Hypergraph<G>>, self => &*self; <'a> &'a Hypergraph<G>
}
impl_traversable! {
    impl for RwLockWriteGuard<'_, Hypergraph<G>>, self => &**self; <'a> &'a Hypergraph<G>
}
impl_traversable! {
    impl for HypergraphRef<G>, self => self.read().unwrap(); <'a> RwLockReadGuard<'a, Hypergraph<G>>
}
impl_traversable! {
    impl for Searcher<G>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph<G>>
}
impl_traversable! {
    impl for &'_ Searcher<G>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph<G>>
}

impl_traversable! {
    impl for Hypergraph<G>, self => self; <'a> &'a Self
}
pub trait TraversableMut: Traversable {
    type GuardMut<'a>:
        TraversableMut<Kind=Self::Kind>
        + Deref<Target=Hypergraph<Self::Kind>>
        + DerefMut where Self: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a>;
}

impl_traversable_mut! {
    impl for Hypergraph<G>, self => self; <'a> &'a mut Self
}
impl_traversable_mut! {
    impl for &'_ mut Hypergraph<G>, self => *self; <'a> &'a mut Hypergraph<G>
}
impl_traversable_mut! {
    impl for RwLockWriteGuard<'_, Hypergraph<G>>, self => &mut **self; <'a> &'a mut Hypergraph<G>
}
impl_traversable_mut! {
    impl for HypergraphRef<G>, self => self.write().unwrap(); <'a> RwLockWriteGuard<'a, Hypergraph<G>>
}

//impl_traversable! {
//    impl for Indexer<G>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<G>>
//}
//impl_traversable! {
//    impl for &'_ mut Indexer<G>,
//    self => self.graph.read().unwrap();
//    <'a> RwLockReadGuard<'a, Hypergraph<G>>
//}
//impl_traversable_mut! {
//    impl for Indexer<G>,
//    self => self.graph.write().unwrap();
//    <'a> RwLockWriteGuard<'a, Hypergraph<G>>
//}
//impl_traversable_mut! {
//    impl for &'_ mut Indexer<G>,
//    self => self.graph.write().unwrap();
//    <'a> RwLockWriteGuard<'a, Hypergraph<G>>
//}
//impl<G: GraphKind, Side: IndexSide<G::Direction>> Traversable for Pather<G, Side> {
//    type Kind = G;
//    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
//    fn graph<'a>(&'a self) -> Self::Guard<'a> {
//        self.indexer.graph()
//    }
//}
//impl<G: GraphKind, Side: IndexSide<G::Direction>> TraversableMut for Pather<G, Side> {
//    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<Self::Kind>> where Side: 'a, Self::Kind: 'a;
//    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
//        self.indexer.graph_mut()
//    }
//}

//impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable for Splitter<T, D, Side> {
//    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<T>> where Side: 'a;
//    fn graph<'a>(&'a self) -> Self::Guard<'a> {
//        self.indexer.graph()
//    }
//}
//impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Splitter<T, D, Side> {
//    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<T>> where Side: 'a;
//    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
//        self.indexer.graph_mut()
//    }
//}
//
//impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable<T> for Contexter<T, D, Side> {
//    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<T>> where Side: 'a;
//    fn graph<'a>(&'a self) -> Self::Guard<'a> {
//        self.indexer.graph()
//    }
//}
//
//impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Contexter<T, D, Side> {
//    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<T>> where Side: 'a;
//    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
//        self.indexer.graph_mut()
//    }
//}

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
