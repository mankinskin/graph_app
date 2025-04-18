use crate::graph::getters::vertex::VertexSet;
use crate::graph::Hypergraph;
use crate::graph::kind::GraphKind;
use crate::graph::vertex::has_vertex_index::HasVertexIndex;
use crate::graph::vertex::parent::Parent;
use crate::graph::vertex::VertexParents;
use crate::graph::getters::ErrorReason;

impl<G: GraphKind> Hypergraph<G> {
    #[track_caller]
    pub fn expect_parent(
        &self,
        index: impl HasVertexIndex,
        parent: impl HasVertexIndex,
    ) -> &Parent {
        self.expect_vertex(index.vertex_index()).expect_parent(parent)
    }
    #[track_caller]
    pub fn expect_parent_mut(
        &mut self,
        index: impl HasVertexIndex,
        parent: impl HasVertexIndex,
    ) -> &mut Parent {
        self.expect_vertex_mut(index.vertex_index()).expect_parent_mut(parent)
    }
    #[track_caller]
    pub fn expect_parents(
        &self,
        index: impl HasVertexIndex,
    ) -> &VertexParents {
        self.expect_vertex(index.vertex_index()).get_parents()
    }
    #[track_caller]
    pub fn expect_parents_mut(
        &mut self,
        index: impl HasVertexIndex,
    ) -> &mut VertexParents {
        self.expect_vertex_mut(index.vertex_index()).get_parents_mut()
    }
    pub fn get_pattern_parents(
        &self,
        pattern: impl IntoIterator<Item = impl HasVertexIndex>,
        parent: impl HasVertexIndex,
    ) -> Result<Vec<Parent>, ErrorReason> {
        pattern
            .into_iter()
            .map(|index| {
                let vertex = self.expect_vertex(index.vertex_index());
                vertex.get_parent(parent.vertex_index()).cloned()
            })
            .collect()
    }
}
