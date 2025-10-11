use std::fmt::Debug;

use crate::split::{
    context::SplitCacheCtx,
    trace::states::context::SplitTraceStatesCtx,
};
use context_search::*;
use context_trace::*;

use crate::interval::IntervalGraph;

#[derive(Debug, PartialEq, Eq)]
pub struct InitInterval {
    pub root: Child,
    pub cache: TraceCache,
    pub end_bound: usize,
}
impl From<IncompleteState> for InitInterval {
    fn from(state: IncompleteState) -> Self {
        Self {
            cache: state.cache,
            root: state.root.index,
            end_bound: state.end_state.cursor.width(),
        }
    }
}
impl<'a, G: HasGraphMut + 'a> From<(&'a mut G, InitInterval)>
    for IntervalGraph
{
    fn from((trav, init): (&'a mut G, InitInterval)) -> Self {
        let InitInterval {
            root,
            cache,
            end_bound,
            ..
        } = init;
        let ctx = TraceCtx { trav, cache };
        let iter = SplitTraceStatesCtx::new(ctx, root, end_bound);
        Self::from(SplitCacheCtx::init(iter))
    }
}
