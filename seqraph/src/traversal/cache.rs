use std::collections::{HashMap, VecDeque};
use super::*;


/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub(crate) struct BUCacheEntry<Q: TraversalQuery> {
    finished: bool,
    mismatch: bool,
    waiting: VecDeque<TraversalNode<Q>>,
}
impl<Q: TraversalQuery> Default for BUCacheEntry<Q> {
    fn default() -> Self {
        Self {
            finished: false,
            mismatch: false,
            waiting: Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TraversalCache<Q: TraversalQuery> {
    bu: HashMap<usize, BUCacheEntry<Q>>,
}
impl<Q: TraversalQuery> Default for TraversalCache<Q> {
    fn default() -> Self {
        Self {
            bu: Default::default()
        }
    }
}
impl<Q: TraversalQuery> TraversalCache<Q> {
    pub fn bu_mismatch(&mut self, root: usize) -> Option<TraversalNode<Q>> {
        self.bu.get_mut(&root).and_then(|e| {
            e.mismatch = true;
            e.waiting.pop_front()
        })
    }
    pub fn bu_finished(&mut self, root: usize) {
        self.bu.entry(root).and_modify(|e| {
            e.finished = true;
            e.waiting.clear();
        });
    }
    pub fn bu_node(&mut self, last_node: &TraversalNode<Q>, entry: ChildLocation) -> Option<()> {
        self.bu.get_mut(&entry.parent.index)
            .and_then(|entry|
                match (entry.finished, entry.mismatch) {
                    (false, false) => {
                        entry.waiting.push_back(last_node.clone());
                        Some(())
                    }
                    (false, true) => {
                        entry.mismatch = false;
                        None
                    },
                    _ => Some(())
                }
            )
            .or_else(|| {
                self.bu.insert(entry.parent.index, BUCacheEntry::default());
                None
            })
    }
}