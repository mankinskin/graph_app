use std::ops::{
    Deref,
    DerefMut,
};

use crate::{
    graph::{
        Hypergraph,
        kind::GraphKind,
    },
    vertex::{
        child::Child,
        VertexData,
    },
};

pub trait VertexedMut<G: GraphKind>: Vertexed<G> {
    fn vertex_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData<G>
        where
            Self: 'a,
    {
        graph.expect_vertex_data_mut(self.vertex_index())
    }
    fn vertex_ref_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData<G> {
        graph.expect_vertex_data_mut(self.vertex_index())
    }
}

impl<G: GraphKind> VertexedMut<G> for Child {}

impl<G: GraphKind, V: VertexedMut<G>> VertexedMut<G> for &'_ mut V {
    fn vertex_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        self,
        graph: &'a mut R,
    ) -> &'a mut VertexData<G>
        where
            Self: 'a,
    {
        V::vertex_ref_mut(self, graph)
    }
    fn vertex_ref_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
        &'a mut self,
        graph: &'a mut R,
    ) -> &'a mut VertexData<G> {
        V::vertex_ref_mut(*self, graph)
    }
}
//impl<G: GraphKind> VertexedMut<G> for &mut VertexData<G> {
//    fn vertex_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
//        self,
//        _graph: &'a mut R,
//    ) -> &'a mut VertexData<G>
//        where Self: 'a
//    {
//        self
//    }
//    fn vertex_ref_mut<'a, R: Deref<Target=Hypergraph<G>> + DerefMut>(
//        &'a mut self,
//        _graph: &'a mut R,
//    ) -> &'a mut VertexData<G> {
//        *self
//    }
//}

pub trait Vertexed<G: GraphKind>: crate::vertex::indexed::AsChild + Sized {
    fn vertex<'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData<G>
        where
            Self: 'a,
    {
        graph.expect_vertex_data(self.vertex_index())
    }
    fn vertex_ref<'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData<G> {
        graph.expect_vertex_data(self.vertex_index())
    }
}

impl<G: GraphKind> Vertexed<G> for Child {}

impl<G: GraphKind, V: Vertexed<G>> Vertexed<G> for &'_ V {
    fn vertex<'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData<G>
        where
            Self: 'a,
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData<G> {
        V::vertex_ref(*self, graph)
    }
}

impl<G: GraphKind, V: Vertexed<G>> Vertexed<G> for &'_ mut V {
    fn vertex<'a, R: Deref<Target=Hypergraph<G>>>(
        self,
        graph: &'a R,
    ) -> &'a VertexData<G>
        where
            Self: 'a,
    {
        V::vertex_ref(self, graph)
    }
    fn vertex_ref<'a, R: Deref<Target=Hypergraph<G>>>(
        &'a self,
        graph: &'a R,
    ) -> &'a VertexData<G> {
        V::vertex_ref(*self, graph)
    }
}

//impl<G: GraphKind> Vertexed<G> for &'_ VertexData {
//    fn vertex<'a, R: Deref<Target=Hypergraph<G>>>(
//        self,
//        _graph: &'a R,
//    ) -> &'a VertexData<G>
//        where Self: 'a
//    {
//        self
//    }
//    fn vertex_ref<'a, R: Deref<Target=Hypergraph<G>>>(
//        &'a self,
//        _graph: &'a R,
//    ) -> &'a VertexData<G> {
//        *self
//    }
//}
//impl<G: GraphKind> Vertexed<G> for &'_ mut VertexData {
//    fn vertex<'a, R: Deref<Target=Hypergraph<G>>>(
//        self,
//        _graph: &'a R,
//    ) -> &'a VertexData<G>
//        where Self: 'a
//    {
//        self
//    }
//    fn vertex_ref<'a, R: Deref<Target=Hypergraph<G>>>(
//        &'a self,
//        _graph: &'a R,
//    ) -> &'a VertexData<G> {
//        *self
//    }
//}
