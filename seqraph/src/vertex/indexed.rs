use crate::traversal::traversable::Traversable;

use super::*;
use std::ops::{
    Deref,
    DerefMut,
};

pub trait VertexedMut: Vertexed {
    fn vertex_mut<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData
        where Self: 'a
    {
        graph.expect_vertex_data_mut(self.index())
    }
    fn vertex_ref_mut<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData {
        graph.expect_vertex_data_mut(self.index())
    }
}
impl VertexedMut for Child {}
impl<V: VertexedMut<>> VertexedMut for &'_ mut V {
    fn vertex_mut<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData
        where Self: 'a
    {
        V::vertex_ref_mut(self, graph)
    }
    fn vertex_ref_mut<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData {
        V::vertex_ref_mut(*self, graph)
    }
}

pub trait Vertexed: AsChild + Sized {
    fn vertex<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        graph.expect_vertex_data(self.index())
    }
    fn vertex_ref<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        graph.expect_vertex_data(self.index())
    }
}
impl Vertexed for Child {}
impl<V: Vertexed> Vertexed for &'_ V {
    fn vertex<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        V::vertex_ref(*self, graph)
    }
}
impl<V: Vertexed<>> Vertexed<> for &'_ mut V {
    fn vertex<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, T: Tokenize, R: Deref<Target=Hypergraph<T>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        V::vertex_ref(*self, graph)
    }
}

pub trait Indexed: Sized {
    fn index(&self) -> VertexIndex;
    fn expect_child_patterns<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&'a self, trav: &'a Trav) -> ChildPatterns {
        trav.graph().expect_child_patterns(self).clone()
    }
    fn expect_child_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        Trav: Traversable<T>,
    >(&'a self, trav: &'a Trav, pid: PatternId) -> Pattern {
        trav.graph().expect_child_pattern(self, pid).clone()
    }
}
impl<I: Indexed> Indexed for &'_ I {
    fn index(&self) -> VertexIndex {
        (**self).index()
    }
}
impl<I: Indexed> Indexed for &'_ mut I {
    fn index(&self) -> VertexIndex {
        (**self).index()
    }
}
impl Indexed for VertexIndex {
    fn index(&self) -> VertexIndex {
        *self
    }
}

pub trait AsChild: Indexed + Wide + std::fmt::Debug {
    fn as_child(&self) -> Child {
        Child::new(self.index(), self.width())
    }
}
impl<T: Indexed + Wide + std::fmt::Debug> AsChild for T {}

pub trait ToChild: AsChild + Sized + std::fmt::Debug {
    fn to_child(self) -> Child {
        self.as_child()
    }
}
impl<T: AsChild> ToChild for T {}

pub trait MaybeIndexed<T: Tokenize> {
    type Inner: Indexed;
    fn into_inner(self) -> Result<Self::Inner, T>;
}
impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for Result<I, T> {
    type Inner = I;
    fn into_inner(self) -> Result<Self::Inner, T> {
        self
    }
}
//impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for I {
//    type Inner = I;
//    fn into_inner(self) -> Result<Self::Inner, T> {
//        Ok(self)
//    }
//}
