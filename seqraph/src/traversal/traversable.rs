use async_std::sync::{RwLockReadGuard, RwLockWriteGuard};
use crate::{
    *,
    Tokenize,
    Hypergraph,
    HypergraphRef,
};
#[async_trait]
pub trait Traversable<'a: 'g, 'g, T: Tokenize>: 'a + Sized + std::fmt::Debug + Sync + Send + Unpin {
    type Guard: Traversable<'g, 'g, T> + Deref<Target=Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard;
}
#[async_trait]
impl <'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for &'a Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    async fn graph(&'g self) -> Self::Guard {
        self
    }
}
#[async_trait]
impl <'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for &'a mut Hypergraph<T> {
    type Guard = &'g Hypergraph<T>;
    async fn graph(&'g self) -> Self::Guard {
        *self
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for RwLockReadGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    async fn graph(&'g self) -> Self::Guard {
        &*self
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize> Traversable<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type Guard = &'g Hypergraph<T>;
    async fn graph(&'g self) -> Self::Guard {
        &**self
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for HypergraphRef<T> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    async fn graph(&'g self) -> Self::Guard {
        self.read().await
    }
}

#[async_trait]
pub trait TraversableMut<'a: 'g, 'g, T: Tokenize>: Traversable<'a, 'g, T> {
    type GuardMut: TraversableMut<'g, 'g, T> + Deref<Target=Hypergraph<T>> + DerefMut;
    async fn graph_mut(&'g mut self) -> Self::GuardMut;
}
#[async_trait]
impl <'a: 'g, 'g, T: Tokenize + 'a> Traversable<'a, 'g, T> for Hypergraph<T> {
    type Guard = &'g Self;
    async fn graph(&'g self) -> Self::Guard {
        self
    }
}
#[async_trait]
impl <'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for Hypergraph<T> {
    type GuardMut = &'g mut Self;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        self
    }
}
#[async_trait]
impl <'a: 'g, 'g, T: Tokenize> TraversableMut<'a, 'g, T> for &'a mut Hypergraph<T> {
    type GuardMut = &'g mut Hypergraph<T>;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        *self
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize> TraversableMut<'a, 'g, T> for RwLockWriteGuard<'a, Hypergraph<T>> {
    type GuardMut = &'g mut Hypergraph<T>;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        &mut **self
    }
}
#[async_trait]
impl<'a: 'g, 'g, T: Tokenize + 'a> TraversableMut<'a, 'g, T> for HypergraphRef<T> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    async fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.write().await
    }
}