use crate::*;

pub mod cache;
pub use cache::*;

pub mod complete;
pub use complete::*;

pub mod augment;
pub use augment::*;


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatternSplitPos {
    pub inner_offset: Option<NonZeroUsize>,
    pub sub_index: usize,
}
pub type VertexSplitPos = HashMap<PatternId, PatternSplitPos>;
pub trait ToVertexSplitPos {
    fn to_vertex_split_pos(self) -> VertexSplitPos;
}
impl ToVertexSplitPos for VertexSplitPos {
    fn to_vertex_split_pos(self) -> VertexSplitPos {
        self
    }
}
impl ToVertexSplitPos for Vec<SubSplitLocation> {
    fn to_vertex_split_pos(self) -> VertexSplitPos {
        self.into_iter().map(|loc|
            (
                loc.location.pattern_id,
                PatternSplitPos {
                    inner_offset: loc.inner_offset,
                    sub_index: loc.location.sub_index,
                },
            )
        )
        .collect()
    }
}
impl ToVertexSplitPos for OffsetSplits {
    fn to_vertex_split_pos(self) -> VertexSplitPos {
        self.splits
    }
}