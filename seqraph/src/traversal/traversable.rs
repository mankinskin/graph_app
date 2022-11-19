use crate::{
    *,
    Tokenize,
    Hypergraph,
    HypergraphRef,
};

pub trait Traversable<T: Tokenize>: Sized + std::fmt::Debug + Unpin {
    type Guard<'g>: Traversable<T> + Deref<Target=Hypergraph<T>> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g>;
}

impl <T: Tokenize> Traversable<T> for &'_ Hypergraph<T> {
    type Guard<'g> = &'g Hypergraph<T> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self
    }
}

impl <T: Tokenize> Traversable<T> for &'_ mut Hypergraph<T> {
    type Guard<'g> = &'g Hypergraph<T> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        *self
    }
}

impl<T: Tokenize> Traversable<T> for RwLockReadGuard<'_, Hypergraph<T>> {
    type Guard<'g> = &'g Hypergraph<T> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        &*self
    }
}

impl<T: Tokenize> Traversable<T> for RwLockWriteGuard<'_, Hypergraph<T>> {
    type Guard<'g> = &'g Hypergraph<T> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        &**self
    }
}

impl<T: Tokenize> Traversable<T> for HypergraphRef<T> {
    type Guard<'g> = RwLockReadGuard<'g, Hypergraph<T>> where Self: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self.read().unwrap()
    }
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