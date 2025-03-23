use derive_more::derive::{
    Deref,
    DerefMut,
};

use super::{
    cache::{
        position::SplitPositionCache,
        vertex::SplitVertexCache,
    },
    trace::states::SplitTraceStateContext,
    vertex::output::RootNode,
};
use crate::{
    graph::vertex::has_vertex_index::HasVertexIndex,
    interval::SplitCache,
    traversal::{
        cache::{
            entry::position::SubSplitLocation,
            label_key::labelled_key,
        },
        split::trace::SplitTraceState,
        traversable::Traversable,
    },
};

#[derive(Debug, Deref, DerefMut)]
pub struct SplitCacheContext<Trav: Traversable> {
    #[deref]
    #[deref_mut]
    pub states_ctx: SplitTraceStateContext<Trav>,

    pub cache: SplitCache,
}
impl<Trav: Traversable> SplitCacheContext<Trav> {
    pub fn init(mut states_ctx: SplitTraceStateContext<Trav>) -> Self {
        let (offsets, root_mode) = states_ctx.completed_splits::<RootNode>(&states_ctx.ctx.root);
        let pos_splits = states_ctx
            .states
            .leaves
            .collect_leaves(&states_ctx.ctx.root, offsets.clone());
        states_ctx.states.filter_trace_states(
            &states_ctx.ctx.trav,
            &states_ctx.ctx.root,
            pos_splits,
        );
        let root_vertex = SplitVertexCache {
            positions: offsets
                .into_iter()
                .map(|(offset, res)| {
                    (
                        offset,
                        SplitPositionCache::root(res.unwrap_or_else(|location| {
                            vec![SubSplitLocation {
                                location,
                                inner_offset: None,
                            }]
                        })),
                    )
                })
                .collect(),
        };
        let cache = SplitCache::new(
            root_mode,
            FromIterator::from_iter([(
                labelled_key(&states_ctx.ctx.trav, states_ctx.ctx.root),
                root_vertex,
            )]),
        );

        Self { states_ctx, cache }
    }
    pub fn apply_trace_state(
        &mut self,
        state: &SplitTraceState,
    ) {
        let &SplitTraceState {
            index,
            offset,
            prev,
        } = state;
        if let Some(ve) = self.cache.get_mut(&index.vertex_index()) {
            ve.positions
                .entry(offset)
                .and_modify(|pe| {
                    pe.top.insert(prev);
                })
                .or_insert_with(|| self.states_ctx.new_split_position(index, offset, prev));
        } else {
            let vertex = self.new_split_vertex(index, offset, prev);
            self.cache
                .insert(labelled_key(&self.states_ctx.ctx.trav, index), vertex);
        }
    }
}
