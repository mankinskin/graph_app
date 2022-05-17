use super::*;
use std::ops::{
    Deref,
    DerefMut,
};

pub trait VertexedMut<'a, 'g>: Vertexed<'a, 'g> {
    fn vertex_mut<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + DerefMut + 'g>(
        self,
        graph: &'g mut R,
    ) -> &'g mut VertexData {
        graph.expect_vertex_data_mut(self.index())
    }
    fn vertex_ref_mut<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + DerefMut + 'g>(
        &'a mut self,
        graph: &'g mut R,
    ) -> &'g mut VertexData {
        graph.expect_vertex_data_mut(self.index())
    }
}
impl<'a, 'g> VertexedMut<'a, 'g> for Child {}
impl<'a, 'g, V: VertexedMut<'a, 'g>> VertexedMut<'a, 'g> for &'a mut V {
    fn vertex_mut<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + DerefMut + 'g>(
        self,
        graph: &'g mut R,
    ) -> &'g mut VertexData {
        V::vertex_ref_mut(self, graph)
    }
    fn vertex_ref_mut<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + DerefMut + 'g>(
        &'a mut self,
        graph: &'g mut R,
    ) -> &'g mut VertexData {
        V::vertex_ref_mut(*self, graph)
    }
}

pub trait Vertexed<'a, 'g>: AsChild + Sized {
    fn vertex<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + 'g>(
        self,
        graph: &'g R,
    ) -> &'g VertexData {
        graph.expect_vertex_data(self.index())
    }
    fn vertex_ref<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + 'g>(
        &'a self,
        graph: &'g R,
    ) -> &'g VertexData {
        graph.expect_vertex_data(self.index())
    }
}
impl<'a, 'g> Vertexed<'a, 'g> for Child {}
impl<'a, 'g, V: Vertexed<'a, 'g>> Vertexed<'a, 'g> for &'a V {
    fn vertex<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + 'g>(
        self,
        graph: &'g R,
    ) -> &'g VertexData {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + 'g>(
        &'a self,
        graph: &'g R,
    ) -> &'g VertexData {
        V::vertex_ref(*self, graph)
    }
}
impl<'a, 'g, V: Vertexed<'a, 'g>> Vertexed<'a, 'g> for &'a mut V {
    fn vertex<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + 'g>(
        self,
        graph: &'g R,
    ) -> &'g VertexData {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<T: Tokenize + 'g, R: Deref<Target=Hypergraph<T>> + 'g>(
        &'a self,
        graph: &'g R,
    ) -> &'g VertexData {
        V::vertex_ref(*self, graph)
    }
}
pub trait Indexed {
    fn index(&self) -> VertexIndex;
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

pub trait AsChild: Indexed + Wide {
    fn as_child(&self) -> Child {
        Child::new(self.index(), self.width())
    }
}
impl<T: Indexed + Wide> AsChild for T {}

pub trait ToChild: AsChild + Sized {
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
