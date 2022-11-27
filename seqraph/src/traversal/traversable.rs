use crate::*;

macro_rules! impl_traversable {
    {
        impl $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? for $target:ty, $self_:ident => $func:expr; <$lt:lifetime> $guard:ty
    } => {
        impl <T: Tokenize $( $(, $par $(: $bhead $( + $btail )* )? ),* )?> Traversable<T> for $target {
            type Guard<$lt> = $guard where Self: $lt;
            fn graph<'g>(&'g $self_) -> Self::Guard<'g> {
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
            fn graph_mut<'g>(&'g mut $self_) -> Self::GuardMut<'g> {
                $func
            }
        }
    }
}
pub(crate) use impl_traversable;
pub(crate) use impl_traversable_mut;

pub trait Traversable<T: Tokenize>: Sized + std::fmt::Debug + Unpin {
    type Guard<'g>: Traversable<T> + Deref<Target=Hypergraph<T>> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g>;
}

impl_traversable! {
    impl for &'_ Hypergraph<T>, self => self; <'g> &'g Hypergraph<T>
}
impl_traversable! {
    impl for &'_ mut Hypergraph<T>, self => *self; <'g> &'g Hypergraph<T>
}
impl_traversable! {
    impl for RwLockReadGuard<'_, Hypergraph<T>>, self => &*self; <'g> &'g Hypergraph<T>
}
impl_traversable! {
    impl for RwLockWriteGuard<'_, Hypergraph<T>>, self => &**self; <'g> &'g Hypergraph<T>
}
impl_traversable! {
    impl for HypergraphRef<T>, self => self.read().unwrap(); <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
impl_traversable! {
    impl <D: MatchDirection> for Searcher<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
impl_traversable! {
    impl<D: MatchDirection> for &'_ Searcher<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}

impl_traversable! {
    impl <D: IndexDirection> for Reader<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
impl_traversable! {
    impl<D: IndexDirection> for &'_ Reader<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
impl_traversable! {
    impl<D: IndexDirection> for &'_ mut Reader<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
impl_traversable! {
    impl for Hypergraph<T>, self => self; <'g> &'g Self
}
impl_traversable! {
    impl <D: IndexDirection> for Indexer<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
impl_traversable! {
    impl<D: IndexDirection> for &'_ mut Indexer<T, D>,
    self => self.graph.read().unwrap();
    <'g> RwLockReadGuard<'g, Hypergraph<T>>
}
pub trait TraversableMut<T: Tokenize>: Traversable<T> {
    type GuardMut<'g>: TraversableMut<T> + Deref<Target=Hypergraph<T>> + DerefMut where Self: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g>;
}

impl_traversable_mut! {
    impl for Hypergraph<T>, self => self; <'g> &'g mut Self
}
impl_traversable_mut! {
    impl for &'_ mut Hypergraph<T>, self => *self; <'g> &'g mut Hypergraph<T>
}
impl_traversable_mut! {
    impl for RwLockWriteGuard<'_, Hypergraph<T>>, self => &mut **self; <'g> &'g mut Hypergraph<T>
}
impl_traversable_mut! {
    impl for HypergraphRef<T>, self => self.write().unwrap(); <'g> RwLockWriteGuard<'g, Hypergraph<T>>
}
impl_traversable_mut! {
    impl<D: IndexDirection> for Reader<T, D>,
    self => self.graph.write().unwrap();
    <'g> RwLockWriteGuard<'g, Hypergraph<T>>
}
impl_traversable_mut! {
    impl<D: IndexDirection> for &'_ mut Reader<T, D>,
    self => self.graph.write().unwrap();
    <'g> RwLockWriteGuard<'g, Hypergraph<T>>
}
impl_traversable_mut! {
    impl<D: IndexDirection> for Indexer<T, D>,
    self => self.graph.write().unwrap();
    <'g> RwLockWriteGuard<'g, Hypergraph<T>>
}
impl_traversable_mut! {
    impl<D: IndexDirection> for &'_ mut Indexer<T, D>,
    self => self.graph.write().unwrap();
    <'g> RwLockWriteGuard<'g, Hypergraph<T>>
}