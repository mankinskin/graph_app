use std::collections::BinaryHeap;
use super::*;


type HashMap<K, V> = DeterministicHashMap<K, V>;
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub(crate) struct BUCacheEntry<Q: TraversalQuery> {
    finished: bool,
    mismatch: bool,
    waiting: BinaryHeap<WaitingNode<Q>>,
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
/// ordered according to priority
#[derive(Clone, Debug, Eq)]
struct WaitingNode<Q: TraversalQuery>(usize, TraversalNode<Q>);

impl<Q: TraversalQuery> PartialEq for WaitingNode<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<Q: TraversalQuery> Ord for WaitingNode<Q> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0)
            .unwrap_or(Ordering::Equal)
    }
}
impl<Q: TraversalQuery> PartialOrd for WaitingNode<Q> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(Ordering::reverse)
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
            e.waiting.pop().map(|w| w.1)
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
            .and_then(|e|
                match (e.finished, e.mismatch) {
                    (false, false) => {
                        e.waiting.push(WaitingNode(entry.sub_index, last_node.clone()));
                        Some(())
                    }
                    (false, true) => {
                        e.mismatch = false;
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