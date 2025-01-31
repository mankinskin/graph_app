use std::borrow::Borrow;

use tracing::instrument;

use crate::{
    insert::HasInsertContext,
    read::{
        overlap::{
            chain::OverlapChain,
            OverlapLink,
        },
        ReadContext,
    },
};
use hypercontext_api::{
    direction::insert::InsertDirection,
    graph::{
        kind::DefaultDirection,
        vertex::pattern::Pattern,
    },
    split::side::SplitBack,
};

impl ReadContext {
    //#[instrument(skip(self, overlaps, link))]
    pub fn back_context_from_path(
        &mut self,
        overlaps: &mut OverlapChain,
        link: &OverlapLink,
    ) -> Pattern {
        let (inner_back_ctx, _loc) = self
            .contexter::<SplitBack>()
            .try_context_path(
                //link.postfix_path.clone().into_context_path(),
                // FIXME: maybe mising root!!!
                link.postfix_path.clone().sub_path,
                //link.overlap,
            )
            .unwrap();

        let back_ctx = if let Some((_, last)) = overlaps.chain.iter_mut().last() {
            self.graph
                .index_pattern(last.band.back_context.borrow())
                .ok()
                //Some(self.graph.read_pattern(last.band.back_context.borrow()))
                .map(|(back_ctx, _)| {
                    last.band.back_context = vec![back_ctx];
                    last.band.back_context.borrow()
                })
        } else {
            None
        }
        .unwrap_or_default();
        DefaultDirection::context_then_inner(back_ctx, inner_back_ctx)
    }
    #[instrument(skip(self, start_bound, overlaps))]
    pub fn take_past_context_pattern(
        &mut self,
        start_bound: usize,
        overlaps: &mut OverlapChain,
    ) -> Option<(usize, Pattern)> {
        let mut past = overlaps.take_past(start_bound);
        match past.chain.len() {
            0 => None,
            1 => {
                let (end_bound, past) = past.chain.pop_last().unwrap();
                Some((end_bound, past.band.into_pattern(self)))
            }
            _ => Some((
                *past.chain.keys().last().unwrap(),
                vec![past.close(self).unwrap()],
            )),
        }
    }
}
