use crate::{
    graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        wide::Wide,
    },
    split::cache::builder::SplitCacheBuilder,
    traversal::{
        cache::entry::NodeType,
        fold::state::FoldState,
        traversable::Traversable,
    },
};

impl SplitCacheBuilder {
    pub fn completed_splits<Trav: Traversable, N: NodeType>(
        trav: &Trav,
        fold_state: &FoldState,
        index: &Child,
    ) -> N::CompleteSplitOutput {
        fold_state
            .cache
            .entries
            .get(&index.vertex_index())
            .map(|e| e.complete_splits::<_, N>(trav, fold_state.end_state.width().into()))
            .unwrap_or_default()
    }
}
