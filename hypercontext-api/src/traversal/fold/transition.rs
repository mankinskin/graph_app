use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::vertex::wide::Wide,
    traversal::fold::FoldContext,
};

use super::TraversalKind;

#[derive(Debug, Deref, DerefMut)]
pub struct TransitionIter<'a, K: TraversalKind> {
    #[deref_mut]
    #[deref]
    pub fctx: &'a mut FoldContext<K>,
}
impl<K: TraversalKind> Iterator for TransitionIter<'_, K> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.fctx.tctx.next().and_then(|(_depth, end_state)| {
            if let Some(end) = end_state {
                assert!(
                    end.width() >= self.max_width,
                    "Parents not evaluated in order"
                );
                let not_final = !end.is_final();
                if end.width() > self.max_width {
                    self.max_width = end.width();
                    self.fctx.end_state = Some(end);
                }
                not_final.then_some(())
            } else {
                Some(())
            }
        })
    }
}
