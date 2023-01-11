use crate::*;

macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <T: Tokenize $( $(, $par $(: $bhead $( + $btail )* )? ),* )?> Traversable<T> for $target {
            type Guard<$lt> = $guard where Self: $lt;
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
        impl <T: Tokenize $( $(, $par $(: $bhead $( + $btail )* )? ),* )?> TraversableMut<T> for $target {
            type GuardMut<$lt> = $guard where Self: $lt;
            fn graph_mut<'a>(&'a mut $self_) -> Self::GuardMut<'a> {
                $func
            }
        }
    }
}
pub(crate) use impl_traversable;
pub(crate) use impl_traversable_mut;

pub trait Traversable<T: Tokenize>: Sized + std::fmt::Debug + Unpin {
    type Guard<'a>: Traversable<T> + Deref<Target=Hypergraph<T>> where Self: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a>;
}

impl_traversable! {
    impl for &'_ Hypergraph<T>, self => self; <'a> &'a Hypergraph<T>
}
impl_traversable! {
    impl for &'_ mut Hypergraph<T>, self => *self; <'a> &'a Hypergraph<T>
}
impl_traversable! {
    impl for RwLockReadGuard<'_, Hypergraph<T>>, self => &*self; <'a> &'a Hypergraph<T>
}
impl_traversable! {
    impl for RwLockWriteGuard<'_, Hypergraph<T>>, self => &**self; <'a> &'a Hypergraph<T>
}
impl_traversable! {
    impl for HypergraphRef<T>, self => self.read().unwrap(); <'a> RwLockReadGuard<'a, Hypergraph<T>>
}
impl_traversable! {
    impl <D: MatchDirection> for Searcher<T, D>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph<T>>
}
impl_traversable! {
    impl<D: MatchDirection> for &'_ Searcher<T, D>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph<T>>
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
impl_traversable! {
    impl for Hypergraph<T>, self => self; <'a> &'a Self
}
impl_traversable! {
    impl <D: IndexDirection> for Indexer<T, D>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph<T>>
}
impl_traversable! {
    impl<D: IndexDirection> for &'_ mut Indexer<T, D>,
    self => self.graph.read().unwrap();
    <'a> RwLockReadGuard<'a, Hypergraph<T>>
}
pub trait TraversableMut<T: Tokenize>: Traversable<T> {
    type GuardMut<'a>: TraversableMut<T> + Deref<Target=Hypergraph<T>> + DerefMut where Self: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a>;
}

impl_traversable_mut! {
    impl for Hypergraph<T>, self => self; <'a> &'a mut Self
}
impl_traversable_mut! {
    impl for &'_ mut Hypergraph<T>, self => *self; <'a> &'a mut Hypergraph<T>
}
impl_traversable_mut! {
    impl for RwLockWriteGuard<'_, Hypergraph<T>>, self => &mut **self; <'a> &'a mut Hypergraph<T>
}
impl_traversable_mut! {
    impl for HypergraphRef<T>, self => self.write().unwrap(); <'a> RwLockWriteGuard<'a, Hypergraph<T>>
}
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
impl_traversable_mut! {
    impl<D: IndexDirection> for Indexer<T, D>,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph<T>>
}
impl_traversable_mut! {
    impl<D: IndexDirection> for &'_ mut Indexer<T, D>,
    self => self.graph.write().unwrap();
    <'a> RwLockWriteGuard<'a, Hypergraph<T>>
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable<T> for Splitter<T, D, Side> {
    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<T>> where Side: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a> {
        self.indexer.graph()
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Splitter<T, D, Side> {
    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<T>> where Side: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
        self.indexer.graph_mut()
    }
}
impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable<T> for Contexter<T, D, Side> {
    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<T>> where Side: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a> {
        self.indexer.graph()
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Contexter<T, D, Side> {
    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<T>> where Side: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
        self.indexer.graph_mut()
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable<T> for Pather<T, D, Side> {
    type Guard<'a> = RwLockReadGuard<'a, Hypergraph<T>> where Side: 'a;
    fn graph<'a>(&'a self) -> Self::Guard<'a> {
        self.indexer.graph()
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Pather<T, D, Side> {
    type GuardMut<'a> = RwLockWriteGuard<'a, Hypergraph<T>> where Side: 'a;
    fn graph_mut<'a>(&'a mut self) -> Self::GuardMut<'a> {
        self.indexer.graph_mut()
    }
}