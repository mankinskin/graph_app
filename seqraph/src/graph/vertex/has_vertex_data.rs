use std::ops::{
    Deref,
    DerefMut,
};

use crate::graph::getters::vertex::VertexSet;
use crate::graph::vertex::child::Child;
use crate::graph::{
    Hypergraph,
    kind::GraphKind,
};
use crate::graph::vertex::data::VertexData;
use crate::graph::vertex::has_vertex_index::ToChild;

pub trait HasVertexDataMut: HasVertexData {
    fn vertex_mut<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData
    where
        Self: 'a,
    {
        graph.expect_vertex_mut(self.vertex_index())
    }
    fn vertex_ref_mut<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData {
        graph.expect_vertex_mut(self.vertex_index())
    }
}

impl HasVertexDataMut for Child {}

impl<V: HasVertexDataMut> HasVertexDataMut for &'_ mut V {
    fn vertex_mut<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData
    where
        Self: 'a,
    {
        V::vertex_ref_mut(self, graph)
    }
    fn vertex_ref_mut<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData {
        V::vertex_ref_mut(*self, graph)
    }
}
//impl<G: GraphKind> VertexedMut<G> for &mut VertexData {
//    fn vertex_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
//        self,
//        _graph: &'a mut R,
//    ) -> &'a mut VertexData
//        where Self: 'a
//    {
//        self
//    }
//    fn vertex_ref_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
//        &'a mut self,
//        _graph: &'a mut R,
//    ) -> &'a mut VertexData {
//        *self
//    }
//}

pub trait HasVertexData: ToChild + Sized {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
    where
        Self: 'a,
    {
        graph.expect_vertex(self.vertex_index())
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        graph.expect_vertex(self.vertex_index())
    }
}

impl HasVertexData for Child {}

impl<V: HasVertexData> HasVertexData for &'_ V {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
    where
        Self: 'a,
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        V::vertex_ref(*self, graph)
    }
}

impl<V: HasVertexData> HasVertexData for &'_ mut V {
    fn vertex<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData
    where
        Self: 'a,
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, G: GraphKind + 'a, R: Deref<Target = Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData {
        V::vertex_ref(*self, graph)
    }
}
