use std::{
    borrow::Borrow,
    fmt::Display,
};

use crate::graph::{
    Hypergraph,
    getters::ErrorReason,
    kind::GraphKind,
    vertex::{
        VertexEntry,
        VertexIndex,
        VertexPatternView,
        child::Child,
        data::VertexData,
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        key::VertexKey,
    },
};

pub trait GetVertexKey {
    fn get_vertex_key<G: GraphKind>(
        &self,
        graph: &Hypergraph<G>,
    ) -> VertexKey;
}
impl<T: GetVertexKey> GetVertexKey for &'_ T {
    fn get_vertex_key<G: GraphKind>(
        &self,
        g: &Hypergraph<G>,
    ) -> VertexKey {
        (*self).get_vertex_key(g)
    }
}
macro_rules! impl_GetVertexKey_with {
    (($g:ident, $_self:ident) => $f:expr, {$($t:ty,)*}) => {
      $(
        impl GetVertexKey for $t {
            fn get_vertex_key<G: GraphKind>(&$_self, $g: &Hypergraph<G>) -> VertexKey {
                $f
            }
        }
      )*
    };
}
macro_rules! impl_GetVertexIndex_with {
    (($g:ident, $_self:ident) => $f:expr, {$($t:ty,)*}) => {
      $(
        impl GetVertexIndex for $t {
            fn get_vertex_index<G: GraphKind>(&$_self, $g: &Hypergraph<G>) -> VertexIndex {
                $f
            }
        }
      )*
    };
}
impl_GetVertexKey_with!(
    (graph, self) =>
        graph.expect_key_for_index(self),
    {
        VertexIndex,
        Child,
    }
);
impl_GetVertexKey_with!(
    (_graph, self) =>
        self.vertex_key(),
    {
        VertexKey,
    }
);
impl_GetVertexIndex_with!(
    (_graph, self) => self.vertex_index(),
    {
        VertexIndex,
        Child,
    }
);

pub trait GetVertexIndex: GetVertexKey + Display + Clone + Copy {
    fn get_vertex_index<G: GraphKind>(
        &self,
        graph: &Hypergraph<G>,
    ) -> VertexIndex;
}
//impl<T: HasVertexIndex + GetVertexKey + Display + Clone + Copy> GetVertexIndex for T {
//    fn get_vertex_index<G: GraphKind>(&self, _: &Hypergraph<G>) -> VertexIndex {
//        self.vertex_index()
//    }
//}
impl GetVertexIndex for VertexKey {
    fn get_vertex_index<G: GraphKind>(
        &self,
        graph: &Hypergraph<G>,
    ) -> VertexIndex {
        graph.expect_index_for_key(self)
    }
}
impl<T: GetVertexIndex> GetVertexIndex for &'_ T {
    fn get_vertex_index<G: GraphKind>(
        &self,
        g: &Hypergraph<G>,
    ) -> VertexIndex {
        (*self).get_vertex_index(g)
    }
}

pub trait VertexSet<I: GetVertexIndex> {
    fn get_vertex(
        &self,
        key: I,
    ) -> Result<&VertexData, ErrorReason>;
    fn get_vertex_mut(
        &mut self,
        key: I,
    ) -> Result<&mut VertexData, ErrorReason>;
    fn expect_vertex(
        &self,
        index: I,
    ) -> &VertexData {
        self.get_vertex(index)
            .unwrap_or_else(|_| panic!("Vertex {} does not exist!", index))
    }
    #[track_caller]
    fn expect_vertex_mut(
        &mut self,
        index: I,
    ) -> &mut VertexData {
        self.get_vertex_mut(index)
            .unwrap_or_else(|_| panic!("Vertex {} does not exist!", index))
    }
    fn vertex_entry(
        &mut self,
        index: I,
    ) -> VertexEntry<'_>;
    fn get_vertices(
        &self,
        indices: impl Iterator<Item = I>,
    ) -> Result<VertexPatternView<'_>, ErrorReason> {
        indices.map(move |index| self.get_vertex(index)).collect()
    }
    #[track_caller]
    fn expect_vertices(
        &self,
        indices: impl Iterator<Item = I>,
    ) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex(index))
            .collect()
    }
    #[track_caller]
    fn contains_vertex(
        &self,
        key: I,
    ) -> bool {
        self.get_vertex(key).is_ok()
    }
}
impl<'t, G: GraphKind, I: GetVertexIndex> VertexSet<&'t I> for Hypergraph<G>
where
    Hypergraph<G>: VertexSet<I>,
{
    fn get_vertex(
        &self,
        key: &'t I,
    ) -> Result<&VertexData, ErrorReason> {
        self.get_vertex(*key)
    }
    fn get_vertex_mut(
        &mut self,
        key: &'t I,
    ) -> Result<&mut VertexData, ErrorReason> {
        self.get_vertex_mut(*key)
    }
    fn vertex_entry(
        &mut self,
        index: &'t I,
    ) -> VertexEntry<'_> {
        self.vertex_entry(*index)
    }
}
impl<G: GraphKind> VertexSet<VertexKey> for Hypergraph<G> {
    fn get_vertex(
        &self,
        key: VertexKey,
    ) -> Result<&VertexData, ErrorReason> {
        self.graph
            .get(key.borrow())
            .ok_or(ErrorReason::UnknownIndex)
    }
    fn get_vertex_mut(
        &mut self,
        key: VertexKey,
    ) -> Result<&mut VertexData, ErrorReason> {
        self.graph
            .get_mut(key.borrow())
            .ok_or(ErrorReason::UnknownIndex)
    }
    fn vertex_entry(
        &mut self,
        index: VertexKey,
    ) -> VertexEntry<'_> {
        self.graph.entry(index)
    }
}
impl<G: GraphKind> VertexSet<VertexIndex> for Hypergraph<G> {
    fn get_vertex(
        &self,
        key: VertexIndex,
    ) -> Result<&VertexData, ErrorReason> {
        self.graph
            .get_index(*key.borrow())
            .map(|(_, d)| d)
            .ok_or(ErrorReason::UnknownIndex)
    }
    fn get_vertex_mut(
        &mut self,
        key: VertexIndex,
    ) -> Result<&mut VertexData, ErrorReason> {
        self.graph
            .get_index_mut(*key.borrow())
            .map(|(_, d)| d)
            .ok_or(ErrorReason::UnknownIndex)
    }
    fn vertex_entry(
        &mut self,
        index: VertexIndex,
    ) -> VertexEntry<'_> {
        let key = *self.graph.get_index(index).unwrap().0;
        self.vertex_entry(key)
    }
}
impl<G: GraphKind> VertexSet<Child> for Hypergraph<G> {
    fn get_vertex(
        &self,
        key: Child,
    ) -> Result<&VertexData, ErrorReason> {
        self.get_vertex(key.vertex_index())
    }
    fn get_vertex_mut(
        &mut self,
        key: Child,
    ) -> Result<&mut VertexData, ErrorReason> {
        self.get_vertex_mut(key.vertex_index())
    }
    fn vertex_entry(
        &mut self,
        index: Child,
    ) -> VertexEntry<'_> {
        let key = *self.graph.get_index(index.vertex_index()).unwrap().0;
        self.vertex_entry(key)
    }
}
impl<G: GraphKind> Hypergraph<G> {
    pub fn get_index_for_key(
        &self,
        key: &VertexKey,
    ) -> Result<VertexIndex, ErrorReason> {
        self.graph.get_index_of(key).ok_or(ErrorReason::UnknownKey)
    }
    #[track_caller]
    pub fn expect_index_for_key(
        &self,
        key: &VertexKey,
    ) -> VertexIndex {
        self.get_index_for_key(key).expect("Key does not exist")
    }
    pub fn get_key_for_index(
        &self,
        index: impl HasVertexIndex,
    ) -> Result<VertexKey, ErrorReason> {
        self.graph
            .get_index(index.vertex_index())
            .map(|(k, _)| *k)
            .ok_or(ErrorReason::UnknownKey)
    }
    #[track_caller]
    pub fn expect_key_for_index(
        &self,
        index: impl HasVertexIndex,
    ) -> VertexKey {
        self.get_key_for_index(index).expect("Key does not exist")
    }

    pub fn next_vertex_index(&self) -> VertexIndex {
        self.graph.len()
    }
    pub fn vertex_iter(
        &self
    ) -> impl Iterator<Item = (&VertexKey, &VertexData)> {
        self.graph.iter()
    }
    pub fn vertex_iter_mut(
        &mut self
    ) -> impl Iterator<Item = (&VertexKey, &mut VertexData)> {
        self.graph.iter_mut()
    }
    pub fn vertex_data_iter(&self) -> impl Iterator<Item = &VertexData> {
        self.graph.values()
    }
    pub fn vertex_data_iter_mut(
        &mut self
    ) -> impl Iterator<Item = &mut VertexData> {
        self.graph.values_mut()
    }
}
