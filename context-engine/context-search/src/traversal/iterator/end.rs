//use super::r#match::{
//    MatchContext,
//    MatchIterator,
//};
use crate::traversal::{
    iterator::r#match::{
        RootSearchIterator,
        TraceNode::Parent,
    },
    state::end::{
        EndState,
        TraceStart,
    },
    MatchContext,
    TraversalKind,
};
use context_trace::trace::{
    traceable::Traceable,
    TraceContext,
};
use derive_new::new;

#[derive(Debug, new)]
pub struct EndIterator<K: TraversalKind>(
    pub TraceContext<K::Trav>,
    pub MatchContext,
);

impl<K: TraversalKind> EndIterator<K> {
    pub fn find_next(&mut self) -> Option<EndState> {
        self.find_map(|flow| Some(flow))
    }
}
impl<K: TraversalKind> Iterator for EndIterator<K> {
    type Item = EndState;

    fn next(&mut self) -> Option<Self::Item> {
        match RootSearchIterator::<K>::new(&self.0.trav, &mut self.1)
            .find_root_cursor()
        {
            Some(root_cursor) => Some({
                let end = match root_cursor.find_end() {
                    Ok(end) => end,
                    Err(root_cursor) =>
                        match root_cursor.next_parents::<K>(&self.0.trav) {
                            Err(end) => end,
                            Ok((parent, batch)) => {
                                assert!(!batch.is_empty());
                                // next batch
                                self.1.nodes.extend(
                                    batch.parents.into_iter().map(Parent),
                                );
                                EndState::mismatch(&self.0.trav, parent)
                            },
                        },
                };
                //debug!("End {:#?}", end);
                TraceStart(&end).trace(&mut self.0);
                end
            }),
            None => None,
        }
    }
}
