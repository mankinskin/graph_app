use std::collections::BTreeSet;

use super::context::SplitCacheContext;
use crate::{
    graph::vertex::{
        child::{
            Child,
            ChildWidth,
        },
        wide::Wide,
    },
    interval::IntervalGraph,
    traversal::traversable::Traversable,
};

#[derive(Debug)]
pub struct SplitRunStep;

#[derive(Debug)]
pub struct SplitRun<Trav: Traversable> {
    ctx: SplitCacheContext<Trav>,
    incomplete: BTreeSet<Child>,
}
impl<'a, Trav: Traversable + 'a> SplitRun<Trav> {
    pub fn init(&mut self) {
        self.ctx
            .cache
            .augment_root(&self.ctx.states_ctx.trav, self.ctx.root);
    }
    pub fn finish(mut self) -> SplitCacheContext<Trav> {
        self.ctx
            .cache
            .augment_nodes(&mut self.ctx.states_ctx, self.incomplete);
        self.ctx
    }
}
impl<'a, Trav: Traversable + 'a> Iterator for SplitRun<Trav> {
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
impl<'a, Trav: Traversable + 'a> From<SplitCacheContext<Trav>> for SplitRun<Trav> {
    fn from(ctx: SplitCacheContext<Trav>) -> Self {
        Self {
            ctx,
            incomplete: Default::default(),
        }
    }
}
impl<'a, Trav: Traversable + 'a> From<SplitCacheContext<Trav>> for IntervalGraph {
    fn from(cache: SplitCacheContext<Trav>) -> Self {
        Self::from(SplitRun::from(cache))
    }
}
impl<'a, Trav: Traversable + 'a> From<SplitRun<Trav>> for IntervalGraph {
    fn from(mut run: SplitRun<Trav>) -> Self {
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
