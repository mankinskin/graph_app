use std::collections::BinaryHeap;
use super::*;


#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub root: usize,
    pub token_pos: usize,
}
pub trait GetCacheKey {
    fn cache_key(&self) -> CacheKey;
}

type HashMap<K, V> = DeterministicHashMap<K, V>;


/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct BUPositionCache<R: ResultKind + Eq, Q: TraversalQuery> {
    state: CacheState,
    waiting: BinaryHeap<WaitingNode<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Default for BUPositionCache<R, Q> {
    fn default() -> Self {
        Self {
            state: CacheState::Waiting,
            waiting: Default::default()
        }
    }
}
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct BUVertexCache<R: ResultKind + Eq, Q: TraversalQuery> {
    num_patterns: usize,
    positions: HashMap<usize, BUPositionCache<R, Q>>
}
#[derive(Clone, Debug)]
pub enum CacheState {
    Mismatch,
    Waiting,
    AtEnd,
}
/// ordered according to priority
#[derive(Clone, Debug, Eq)]
pub struct WaitingNode<R: ResultKind + Eq, Q: TraversalQuery> {
    sub_index: usize,
    node: ParentNode<R, Q>,
}

impl<R: ResultKind + Eq, Q: TraversalQuery> PartialEq for WaitingNode<R, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.sub_index.eq(&other.sub_index)
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Ord for WaitingNode<R, Q> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sub_index.partial_cmp(&other.sub_index)
            .unwrap_or(Ordering::Equal)
    }
}
impl<R: ResultKind + Eq, Q: TraversalQuery> PartialOrd for WaitingNode<R, Q> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sub_index.partial_cmp(&other.sub_index).map(Ordering::reverse)
    }
}

#[derive(Clone, Debug)]
pub struct TraversalCache<R: ResultKind + Eq, Q: TraversalQuery> {
    bu: HashMap<usize, BUVertexCache<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> Default for TraversalCache<R, Q> {
    fn default() -> Self {
        Self {
            bu: Default::default()
        }
    }
}
#[derive(Clone, Debug)]
pub enum OnParent<R: ResultKind + Eq, Q: TraversalQuery> {
    First,
    Waiting,
    Last(Option<BinaryHeap<WaitingNode<R, Q>>>),
}
#[derive(Clone, Debug)]
pub enum OnIndexEnd<R: ResultKind + Eq, Q: TraversalQuery> {
    Waiting,
    Finished(BinaryHeap<WaitingNode<R, Q>>),
}
impl<R: ResultKind + Eq, Q: TraversalQuery> TraversalCache<R, Q> {
    pub fn get_entry_mut(&mut self, key: CacheKey) -> Option<&mut BUPositionCache<R, Q>> {
        self.bu.get_mut(&key.root)
            .and_then(|e|
                e.positions.get_mut(&key.token_pos)
            )
    }
    /// adds node to cache and returns the state of the insertion
    pub fn on_parent_node(&mut self, node: ParentNode<R, Q>) -> OnParent<R, Q> {
        let key = node.path.cache_key();
        let ChildLocation {
            sub_index,
            ..
        } = node.path.entry();
        if let Some(ve) = self.bu.get_mut(&key.root) {
            if let Some(e) = ve.positions.get_mut(&key.token_pos) {
                e.waiting.push(WaitingNode {
                    sub_index,
                    node: node.clone()
                });
                assert!(e.waiting.len() <= ve.num_patterns);
                if e.waiting.len() == ve.num_patterns {
                    OnParent::Last(
                        matches!(
                            e.state,
                            CacheState::AtEnd
                        ).then(|| e.waiting)
                    )
                } else {
                    OnParent::Waiting
                }
            } else {
                // new position
                ve.positions.insert(key.token_pos, BUPositionCache::default());
                OnParent::First
            }
        } else {
            // new vertex, with position
            let mut positions: HashMap<_, _> = Default::default();
            positions.insert(key.token_pos, BUPositionCache::default());
            self.bu.insert(key.root, BUVertexCache {
                num_patterns: node.num_patterns,
                positions
            });
            OnParent::First
        }
    }
    /// triggered when a path finds a mismatch
    pub fn on_bu_mismatch(&mut self, key: CacheKey) -> Option<ParentNode<R, Q>> {
        self.get_entry_mut(key).and_then(|e| {
            e.state = CacheState::Mismatch;
            e.waiting.pop().map(|w| w.node)
        })
    }
    /// triggered when a path reached the end of an index
    pub fn bu_index_end(&mut self, key: CacheKey) -> OnIndexEnd<R, Q> {
        let mut ve = self.bu.get_mut(&key.root).unwrap();
        let mut e = ve.positions.get_mut(&key.token_pos).unwrap();
        if e.waiting.len() == ve.num_patterns {
            drop(e);
            let e = ve.positions.remove(&key.token_pos).unwrap();
            OnIndexEnd::Finished(e.waiting)
        } else {
            assert!(e.waiting.len() < ve.num_patterns);
            e.state = CacheState::AtEnd;
            OnIndexEnd::Waiting
        }
    }
}