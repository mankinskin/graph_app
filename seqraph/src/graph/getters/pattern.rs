use crate::graph::getters::vertex::VertexSet;
use crate::graph::vertex::ChildPatterns;
use crate::graph::Hypergraph;
use crate::graph::kind::GraphKind;
use crate::graph::vertex::has_vertex_index::HasVertexIndex;
use crate::graph::vertex::location::pattern::IntoPatternLocation;
use crate::graph::vertex::pattern::{Pattern, pattern_width};
use crate::graph::vertex::pattern::id::PatternId;
use crate::graph::vertex::pattern::pattern_range::PatternRangeIndex;
use crate::search::NoMatch;

impl<G: GraphKind> Hypergraph<G> {
    pub fn get_pattern_at(
        &self,
        location: impl IntoPatternLocation,
    ) -> Result<&Pattern, NoMatch> {
        let location = location.into_pattern_location();
        let vertex = self.get_vertex(location.parent)?;
        let child_patterns = vertex.get_child_patterns();
        child_patterns
            .get(&location.id)
            .ok_or(NoMatch::NoChildPatterns) // todo: better error
    }
    #[track_caller]
    pub fn expect_pattern_at(
        &self,
        location: impl IntoPatternLocation,
    ) -> &Pattern {
        let location = location.into_pattern_location();
        self.get_pattern_at(location)
            .unwrap_or_else(|_| panic!("Pattern not found at location {:#?}", location))
    }
    pub fn get_child_patterns_of(
        &self,
        index: impl HasVertexIndex,
    ) -> Result<&ChildPatterns, NoMatch> {
        self.get_vertex(index.vertex_index())
            .map(|vertex| vertex.get_child_patterns())
    }
    pub fn get_pattern_of(
        &self,
        index: impl HasVertexIndex,
        pid: PatternId,
    ) -> Result<&Pattern, NoMatch> {
        self.get_vertex(index.vertex_index())
            .and_then(|vertex| vertex.get_child_pattern(&pid))
    }
    #[track_caller]
    pub fn expect_child_pattern(
        &self,
        index: impl HasVertexIndex,
        pid: PatternId,
    ) -> &Pattern {
        self.expect_vertex(index.vertex_index()).expect_child_pattern(&pid)
    }
    #[track_caller]
    pub fn expect_child_patterns(
        &self,
        index: impl HasVertexIndex,
    ) -> &ChildPatterns {
        self.expect_vertex(index.vertex_index()).get_child_patterns()
    }

    #[track_caller]
    pub fn expect_any_child_pattern(
        &self,
        index: impl HasVertexIndex,
    ) -> (&PatternId, &Pattern) {
        self.expect_vertex(index.vertex_index()).expect_any_child_pattern()
    }
    #[track_caller]
    pub fn expect_pattern_range_width(
        &self,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex,
    ) -> usize {
        pattern_width(self.expect_pattern_range(location, range))
    }
}
