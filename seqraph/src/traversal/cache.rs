use std::collections::BinaryHeap;
use super::*;


#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub(crate) struct CacheKey {
    pub root: usize,
    pub token_pos: usize,
}
pub(crate) trait GetCacheKey {
    fn cache_key(&self) -> CacheKey;
}
//pub(crate) trait TryCacheKey {
//    fn try_cache_key(&self) -> Option<CacheKey>;
//}

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
    bu: HashMap<CacheKey, BUCacheEntry<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Default for TraversalCache<R, Q> {
    fn default() -> Self {
        Self {
            bu: Default::default()
        }
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> TraversalCache<R, Q> {
    /// triggered on new node
    pub fn on_bu_node(&mut self, primer: &<R as ResultKind>::Primer, last_node: &TraversalNode<R, Q>) -> Option<()> {
        let key = primer.cache_key();
        let sub_index = primer.entry().sub_index;
        self.bu.get_mut(&key)
            .and_then(|e|
                match (e.finished, e.mismatch) {
                    (false, false) => {
                        e.waiting.push(WaitingNode(sub_index, last_node.clone()));
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
                self.bu.insert(key, BUCacheEntry::default());
                None
            })
    }
    /// triggered on mismatch node
    pub fn on_bu_mismatch(&mut self, key: CacheKey) -> Option<TraversalNode<R, Q>> {
        self.bu.get_mut(&key).and_then(|e| {
            e.mismatch = true;
            e.waiting.pop().map(|w| w.1)
        })
    }
    /// triggered at_index_end
    pub fn on_bu_finished(&mut self, key: CacheKey) {
        self.bu.entry(key).and_modify(|e| {
            e.finished = true;
            e.waiting.clear();
        });
    }
}