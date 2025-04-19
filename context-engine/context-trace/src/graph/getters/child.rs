use crate::graph::{
    Hypergraph,
    getters::{
        ErrorReason,
        vertex::VertexSet,
    },
    kind::GraphKind,
    vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        location::child::{
            ChildLocation,
            IntoChildLocation,
        },
        pattern::Pattern,
    },
};

impl<G: GraphKind> Hypergraph<G> {
    pub fn get_child_at(
        &self,
        location: impl IntoChildLocation,
    ) -> Result<&Child, ErrorReason> {
        let location = location.into_child_location();
        let pattern = self.get_pattern_at(location.clone())?;
        pattern
            .get(location.sub_index.clone())
            .ok_or(ErrorReason::NoChildPatterns) // todo: better error
    }
    pub fn get_child_mut_at(
        &mut self,
        location: impl IntoChildLocation,
    ) -> Result<&mut Child, ErrorReason> {
        let location = location.into_child_location();
        let pattern = self.get_pattern_mut_at(location.clone())?;
        pattern
            .get_mut(location.sub_index.clone())
            .ok_or(ErrorReason::NoChildPatterns) // todo: better error
    }
    #[track_caller]
    pub fn expect_child_at(
        &self,
        location: impl IntoChildLocation,
    ) -> &Child {
        let location = location.into_child_location();
        self.get_child_at(location.clone()).unwrap_or_else(|_| {
            panic!("Child not found at location {:#?}", location)
        })
    }
    #[track_caller]
    pub fn expect_child_mut_at(
        &mut self,
        location: impl IntoChildLocation,
    ) -> &mut Child {
        let location = location.into_child_location();
        self.get_child_mut_at(location.clone()).unwrap_or_else(|_| {
            panic!("Child not found at location {:#?}", location)
        })
    }
    pub fn expect_is_at_end(
        &self,
        location: &ChildLocation,
    ) -> bool {
        self.expect_vertex(location.vertex_index())
            .expect_pattern_len(&location.pattern_id)
            == location.sub_index + 1
    }
    pub fn expect_child_offset(
        &self,
        loc: &ChildLocation,
    ) -> usize {
        self.expect_vertex(loc.vertex_index())
            .expect_child_offset(&loc.clone().to_sub_location())
    }
    pub fn expect_child(
        &self,
        index: impl HasVertexIndex,
    ) -> Child {
        self.to_child(index)
    }
    pub fn to_child(
        &self,
        index: impl HasVertexIndex,
    ) -> Child {
        Child::new(index.vertex_index(), self.expect_index_width(&index))
    }
    pub fn to_children(
        &self,
        indices: impl IntoIterator<Item = impl HasVertexIndex>,
    ) -> Pattern {
        indices.into_iter().map(|i| self.to_child(i)).collect()
    }
}
