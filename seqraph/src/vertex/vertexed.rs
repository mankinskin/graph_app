use super::*;
use std::ops::{
    Deref,
    DerefMut,
};

pub trait VertexedMut: Vertexed {
    fn vertex_mut<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData
        where Self: 'a
    {
        graph.expect_vertex_data_mut(self.index())
    }
    fn vertex_ref_mut<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData {
        graph.expect_vertex_data_mut(self.index())
    }
}
impl VertexedMut for Child {}
impl<V: VertexedMut<>> VertexedMut for &'_ mut V {
    fn vertex_mut<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData
        where Self: 'a
    {
        V::vertex_ref_mut(self, graph)
    }
    fn vertex_ref_mut<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData {
        V::vertex_ref_mut(*self, graph)
    }
}
impl VertexedMut for &mut VertexData {
    fn vertex_mut<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        self,
        _graph: &'a mut R,
    ) -> &'a mut VertexData
        where Self: 'a
    {
        self
    }
    fn vertex_ref_mut<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        &'a mut self,
        _graph: &'a mut R,
    ) -> &'a mut VertexData {
        *self
    }
}

pub trait Vertexed: AsChild + Sized {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        graph.expect_vertex_data(self.index())
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        graph.expect_vertex_data(self.index())
    }
}
impl Vertexed for Child {}
impl<V: Vertexed> Vertexed for &'_ V {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        V::vertex_ref(*self, graph)
    }
}
impl<V: Vertexed> Vertexed for &'_ mut V {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        V::vertex_ref(*self, graph)
    }
}

impl Vertexed for &'_ VertexData {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        _graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        self
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        _graph: &'a R,
    ) -> &'a VertexData {
        *self
    }
}
impl Vertexed for &'_ mut VertexData {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        _graph: &'a R,
    ) -> &'a VertexData
        where Self: 'a
    {
        self
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        _graph: &'a R,
    ) -> &'a VertexData {
        *self
    }
}