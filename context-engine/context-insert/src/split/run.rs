use std::collections::BTreeSet;

use super::context::SplitCacheCtx;
use crate::interval::IntervalGraph;
use context_trace::*;

#[derive(Debug)]
pub struct SplitRunStep;

#[derive(Debug)]
pub struct SplitRun<G: HasGraph> {
    ctx: SplitCacheCtx<G>,
    incomplete: BTreeSet<Child>,
}
impl<G: HasGraph> SplitRun<G> {
    pub fn init(&mut self) {
        self.ctx.cache.augment_root(
            &self.ctx.states_ctx.trav,
            self.ctx.states_ctx.ctx.root,
        );
    }
    pub fn finish(mut self) -> SplitCacheCtx<G> {
        self.ctx
            .cache
            .augment_nodes(&mut self.ctx.states_ctx, self.incomplete);
        self.ctx
    }
}
impl<G: HasGraph> Iterator for SplitRun<G> {
    type Item = SplitRunStep;
    fn next(&mut self) -> Option<Self::Item> {
        self.ctx.states_ctx.states.next().map(|state| {
            self.ctx.apply_trace_state(&state);
            self.incomplete.insert(state.index);
            let complete = self
                .incomplete
                .split_off(&ChildWidth(state.index.width() + 1));
            self.ctx
                .cache
                .augment_nodes(&mut self.ctx.states_ctx, complete);
            SplitRunStep
        })
    }
}
impl<G: HasGraph> From<SplitCacheCtx<G>> for SplitRun<G> {
    fn from(ctx: SplitCacheCtx<G>) -> Self {
        Self {
            ctx,
            incomplete: Default::default(),
        }
    }
}
impl<G: HasGraph> From<SplitCacheCtx<G>> for IntervalGraph {
    fn from(cache: SplitCacheCtx<G>) -> Self {
        Self::from(SplitRun::from(cache))
    }
}
impl<G: HasGraph> From<SplitRun<G>> for IntervalGraph {
    fn from(mut run: SplitRun<G>) -> Self {
        run.init();
        run.all(|_| true); // run iterator to end
        let cache = run.finish();
        Self {
            root: cache.states_ctx.ctx.root,
            states: cache.states_ctx.states,
            cache: cache.cache,
        }
    }
}
