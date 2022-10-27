use crate::*;
use super::*;

impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn back_context_from_path(
        &mut self,
        overlaps: &mut OverlapChain,
        link: &OverlapLink,
    ) -> Pattern {
        let (inner_back_ctx, _loc) = self.contexter::<IndexBack>().try_context_path(
            link.postfix_path.clone().into_context_path(),
            link.overlap,
        ).unwrap();
        D::context_then_inner(
            overlaps.path.iter_mut().last()
                .and_then(|(_, last)| {
                    self.graph.index_pattern(last.band.back_context.borrow()).ok()
                        .map(|(back_ctx, _)| back_ctx)
                    //Some(self.graph.read_pattern(last.band.back_context.borrow()))
                        .map(|back_ctx| (last, back_ctx))
                })
                .map(|(last, back_ctx)| {
                    last.band.back_context = vec![back_ctx];
                    last.band.back_context.borrow()
                })
                .unwrap_or_default(),
            inner_back_ctx,
        )
    }
    pub(crate) fn take_past_context_pattern(
        &mut self,
        start_bound: usize,
        overlaps: &mut OverlapChain,
    ) -> Option<(usize, Pattern)> {
        let mut past = overlaps.take_past(start_bound);
        match past.path.len() {
            0 => None,
            1 => {
                let (end_bound, past) = past.path.pop_last().unwrap();
                Some((end_bound, past.band.into_pattern(self)))
            },
            _ => Some((*past.path.keys().last().unwrap(), vec![past.close(self).unwrap()])),
        }
    }
}