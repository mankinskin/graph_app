use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use itertools::Itertools;

use crate::{
    //insert::side::{
    //    IndexBack,
    //    IndexSide,
    //},
    //join::partition::splits::offset::OffsetSplits,
    split::{
        cache::builder::SplitCacheBuilder,
        PatternSplitPos,
    },
};
use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            data::VertexData,
            has_vertex_index::HasVertexIndex,
            location::SubLocation,
            pattern::{
                id::PatternId,
                Pattern,
            },
            wide::Wide,
        },
    },
    traversal::{
        cache::entry::{
            position::SubSplitLocation,
            vertex::VertexCache,
            NodeSplitOutput,
            NodeType,
            Offset,
        },
        fold::state::FoldState,
        traversable::Traversable,
    },
    HashSet,
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