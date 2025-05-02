use crate::traversal::{
    state::end::{
        EndKind,
        EndReason,
        EndState,
    },
    OptGen::{
        self,
        Pass,
        Yield,
    },
    TraversalKind,
};
use derive_new::new;

use super::r#match::{
    MatchContext,
    MatchIterator,
};
#[derive(Debug, new)]
pub struct EndIterator<'a, K: TraversalKind>(&'a K::Trav, &'a mut MatchContext);

impl<'a, K: TraversalKind> Iterator for EndIterator<'a, K> {
    type Item = OptGen<EndState>;

    fn next(&mut self) -> Option<Self::Item> {
        match MatchIterator::<K>::new(self.0, self.1).next() {
            Some(Yield(root_cursor)) => Some(Yield(
                // add cache for path to parent
                // TODO: add cache for end
                match root_cursor.find_end() {
                    Ok(end) => end,
                    Err(root_cursor) => match root_cursor
                        .state
                        .root_parent()
                        .next_parents::<K>(&self.0)
                    {
                        // TODO: if no new batch, return end state
                        Err(end) => end,
                        Ok((parent, batch)) => {
                            assert!(!batch.is_empty());
                            // next batch
                            self.1.batches.push_back(batch);
                            EndState {
                                reason: EndReason::Mismatch,
                                kind: EndKind::from_start_path(
                                    parent.path,
                                    parent.root_pos,
                                    self.0,
                                ),
                                cursor: parent.cursor,
                            }
                        },
                    },
                },
            )),
            Some(Pass) => Some(Pass),
            None => None,
        }
    }
}
