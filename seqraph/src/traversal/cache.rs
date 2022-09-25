use std::collections::BinaryHeap;
use super::*;


type HashMap<K, V> = DeterministicHashMap<K, V>;
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub(crate) struct BUCacheEntry<R: ResultKind + Eq, Q: TraversalQuery> {
    finished: bool,
    mismatch: bool,
    waiting: BinaryHeap<WaitingNode<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Default for BUCacheEntry<R, Q> {
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
struct WaitingNode<R: ResultKind + Eq, Q: TraversalQuery>(usize, TraversalNode<R, Q>);

impl<R: ResultKind + Eq, Q: TraversalQuery> PartialEq for WaitingNode<R, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Ord for WaitingNode<R, Q> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0)
            .unwrap_or(Ordering::Equal)
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> PartialOrd for WaitingNode<R, Q> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(Ordering::reverse)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TraversalCache<R: ResultKind + Eq, Q: TraversalQuery> {
    bu: HashMap<usize, BUCacheEntry<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Default for TraversalCache<R, Q> {
    fn default() -> Self {
        Self {
            bu: Default::default()
        }
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> TraversalCache<R, Q> {
    pub fn bu_mismatch(&mut self, root: usize) -> Option<TraversalNode<R, Q>> {
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
    pub fn bu_node(&mut self, last_node: &TraversalNode<R, Q>, entry: ChildLocation) -> Option<()> {
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