use crate::{
    *,
    Tokenize,
    Hypergraph,
    HypergraphRef,
};

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

pub trait TraversableMut<T: Tokenize>: Traversable<T> {
    type GuardMut<'g>: TraversableMut<T> + Deref<Target=Hypergraph<T>> + DerefMut where Self: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g>;
}

impl <T: Tokenize> Traversable<T> for Hypergraph<T> {
    type Guard<'g> = &'g Self where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self
    }
}

impl <T: Tokenize> TraversableMut<T> for Hypergraph<T> {
    type GuardMut<'g> = &'g mut Self where Self: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g> {
        self
    }
}

impl <T: Tokenize> TraversableMut<T> for &'_ mut Hypergraph<T> {
    type GuardMut<'g> = &'g mut Hypergraph<T> where Self: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g> {
        *self
    }
}

impl<T: Tokenize> TraversableMut<T> for RwLockWriteGuard<'_, Hypergraph<T>> {
    type GuardMut<'g> = &'g mut Hypergraph<T> where Self: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g> {
        &mut **self
    }
}

impl<T: Tokenize> TraversableMut<T> for HypergraphRef<T> {
    type GuardMut<'g> = RwLockWriteGuard<'g, Hypergraph<T>> where Self: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g> {
        self.write().unwrap()
    }
}