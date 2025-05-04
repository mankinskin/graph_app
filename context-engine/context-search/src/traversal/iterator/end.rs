//use super::r#match::{
//    MatchContext,
//    MatchIterator,
//};
use crate::traversal::{
    iterator::r#match::TraceNode::Parent,
    state::{
        end::{
            EndKind,
            EndReason,
            EndState,
        },
        parent::batch::RootSearchIterator,
    },
    MatchContext,
    OptGen::{
        self,
        Pass,
        Yield,
    },
    TraversalKind,
};
use context_trace::trace::{
    traceable::Traceable,
    TraceContext,
};
use derive_new::new;
use tracing::debug;

#[derive(Debug, new)]
pub struct EndIterator<K: TraversalKind>(
    pub TraceContext<K::Trav>,
    pub MatchContext,
);

impl<K: TraversalKind> EndIterator<K> {
    pub fn find_next(&mut self) -> Option<EndState> {
        self.find_map(|flow| match flow {
            Yield(end) => Some(end),
            Pass => None,
        })
    }
}
impl<K: TraversalKind> Iterator for EndIterator<K> {
    type Item = OptGen<EndState>;

    fn next(&mut self) -> Option<Self::Item> {
        match RootSearchIterator::<K>::new(&self.0.trav, &mut self.1)
            .find_root_cursor()
        {
            Some(root_cursor) => Some({
                let end = match root_cursor.find_end() {
                    Ok(end) => end,
                    Err(root_cursor) => match root_cursor
                        .state
                        .root_parent()
                        .next_parents::<K>(&self.0.trav)
                    {
                        Err(end) => end,
                        Ok((parent, batch)) => {
                            assert!(!batch.is_empty());
                            // next batch
                            self.1
                                .nodes
                                .extend(batch.parents.into_iter().map(Parent));
                            EndState {
                                reason: EndReason::Mismatch,
                                kind: EndKind::from_start_path(
                                    parent.path,
                                    parent.root_pos,
                                    &self.0.trav,
                                ),
                                cursor: parent.cursor,
                            }
                        },
                    },
                };
                debug!("End {:#?}", end);
                end.trace(&mut self.0);
                Yield(end)
            }),
            None => None,
        }
    }
}
