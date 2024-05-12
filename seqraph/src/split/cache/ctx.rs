use crate::shared::*;

#[derive(Debug)]
pub struct CacheContext {
    pub leaves: Leaves,
    pub states: VecDeque<TraceState>,
}
impl CacheContext {
    pub fn new_split_position<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: Child,
        offset: NonZeroUsize,
        prev: SplitKey,
    ) -> SplitPositionCache {
        let graph = trav.graph();
        let (_, node) = graph.expect_vertex(index);

        // handle clean splits
        match cleaned_position_splits(
            node.children.iter(),
            offset,
        ) {
            Ok(subs) => {
                let next = self.leaves.filter_trace_states(
                    trav,
                    &index,
                    Vec::from_iter([(offset, subs.clone())]),
                );
                self.states.extend(next);
                SplitPositionCache::new(prev, subs)
            },
            Err(location) => {
                self.leaves.push(SplitKey::new(index, offset));
                SplitPositionCache::new(prev, vec![
                    SubSplitLocation {
                        location,
                        inner_offset: None,
                    }
                ])
            }
        }
    }
}
