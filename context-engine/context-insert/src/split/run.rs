use std::collections::BTreeSet;

use super::context::SplitCacheContext;
use crate::interval::IntervalGraph;
use context_trace::{
    graph::vertex::{
        child::{
            Child,
            ChildWidth,
        },
        wide::Wide,
    },
    traversal::has_graph::HasGraph,
};
#[derive(Debug)]
pub struct SplitRunStep;

#[derive(Debug)]
pub struct SplitRun<G: HasGraph> {
    ctx: SplitCacheContext<G>,
    incomplete: BTreeSet<Child>,
}
impl<'a, G: HasGraph + 'a> SplitRun<G> {
    pub fn init(&mut self) {
        self.ctx
            .cache
            .augment_root(&self.ctx.states_ctx.trav, self.ctx.root);
    }
    pub fn finish(mut self) -> SplitCacheContext<G> {
        self.ctx
            .cache
            .augment_nodes(&mut self.ctx.states_ctx, self.incomplete);
        self.ctx
    }
}
impl<'a, G: HasGraph + 'a> Iterator for SplitRun<G> {
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
impl<'a, G: HasGraph + 'a> From<SplitCacheContext<G>> for SplitRun<G> {
    fn from(ctx: SplitCacheContext<G>) -> Self {
        Self {
            ctx,
            incomplete: Default::default(),
        }
    }
}
impl<'a, G: HasGraph + 'a> From<SplitCacheContext<G>> for IntervalGraph {
    fn from(cache: SplitCacheContext<G>) -> Self {
        Self::from(SplitRun::from(cache))
    }
}
impl<'a, G: HasGraph + 'a> From<SplitRun<G>> for IntervalGraph {
    fn from(mut run: SplitRun<G>) -> Self {
        run.init();
        run.all(|_| true); // run iterator to end
        let cache = run.finish();
        Self {
            root: cache.root,
            states: cache.states_ctx.states,
            cache: cache.cache,
        }
    }
}
