use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use crate::{
    *,
    Tokenize,
    Hypergraph,
    HypergraphRef,
};
pub trait Traversable<'a: 'g, 'g, T: Tokenize>: 'a + Sized + std::fmt::Debug {
    type Guard: Traversable<'g, 'g, T> + Deref<Target=Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard;
}
impl <'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for &'a Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for &'a mut Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        *self
    }
}
impl<'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for RwLockReadGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        &*self
    }
}
impl<'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    fn graph(&'g self) -> Self::Guard {
        &**self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for HypergraphRef<T> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.read().unwrap()
    }
}

pub(crate) trait TraversableMut<'a: 'g, 'g, T: Tokenize> : Traversable<'a, 'g, T> {
    type GuardMut: TraversableMut<'g, 'g, T> + Deref<Target=Hypergraph<T>> + DerefMut;
    fn graph_mut(&'g mut self) -> Self::GuardMut;
}
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for Hypergraph<T> {
    type Guard = &'g Self;
    fn graph(&'g self) -> Self::Guard {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for Hypergraph<T> {
    type GuardMut = &'g mut Self;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self
    }
}
impl <'a: 'g, 'g, T: Tokenize> TraversableMut<'a, 'g, T> for &'a mut Hypergraph<T> {
    type GuardMut = &'g mut Hypergraph<T>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        *self
    }
}
impl<'a: 'g, 'g, T: Tokenize> TraversableMut<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type GuardMut = &'g mut Hypergraph<T>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        &mut **self
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for HypergraphRef<T> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.write().unwrap()
    }
}