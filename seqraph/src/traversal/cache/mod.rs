use std::collections::BinaryHeap;
use super::*;

pub mod node;
pub use node::*;
pub mod key;
pub use key::*;

type HashMap<K, V> = DeterministicHashMap<K, V>;

#[derive(Clone, Debug)]
pub enum OnParent<R: ResultKind + Eq, Q: TraversalQuery> {
    First,
    Waiting,
    Last(Option<BinaryHeap<WaitingNode<R, Q>>>),
}
#[derive(Clone, Debug)]
pub enum OnChild<R: ResultKind + Eq, Q: TraversalQuery> {
    First,
    Waiting,
    Last(Option<BinaryHeap<WaitingNode<R, Q>>>),
}
#[derive(Clone, Debug)]
pub enum OnIndexEnd<R: ResultKind + Eq, Q: TraversalQuery> {
    Waiting,
    Finished(BinaryHeap<WaitingNode<R, Q>>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CacheNode<R: ResultKind + Eq, Q: TraversalQuery> {
    Parent(ParentNode<R, Q>),
    Child(ParentNode<R, Q>),
}
#[derive(Clone, Debug)]
pub enum CacheState {
    Mismatch,
    Waiting,
    AtEnd,
}

/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct BUPositionCache<R: ResultKind + Eq, Q: TraversalQuery> {
    pub state: CacheState,
    pub prev: HashSet<CacheKey>,
    /// leading node (arrived first)
    pub node: TraversalNode<R, Q>,
    /// waiting nodes, continued when mismatch is found
    pub waiting: Vec<TraversalNode<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> BUPositionCache<R, Q> {
    pub fn new(prev: Option<CacheKey>, node: TraversalNode<R, Q>) -> Self {
        Self {
            state: CacheState::Waiting,
            prev: prev.map(|prev| maplit::hashset![prev]).unwrap_or_default(),
            node,
            waiting: vec![],
        }
    }
}
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct BUVertexCache<R: ResultKind + Eq, Q: TraversalQuery> {
    positions: HashMap<usize, BUPositionCache<R, Q>>
}
#[derive(Clone, Debug)]
pub struct TraversalCache<R: ResultKind + Eq, Q: TraversalQuery> {
    bu: HashMap<usize, BUVertexCache<R, Q>>,
}
impl<R: ResultKind + Eq, Q: TraversalQuery> TraversalCache<R, Q> {
    pub fn new(index: usize, query: Q) -> (Self, CacheKey) {
        let bu = Default::default();
        let s = Self {
            bu,
        };
        let k = s.add_node(None, TraversalNode::Start(index, query)).unwrap();
        (s, k)
    }
    pub fn get_entry_mut(&mut self, key: CacheKey) -> Option<&mut BUPositionCache<R, Q>> {
        self.bu.get_mut(&key.root)
            .and_then(|e|
                e.positions.get_mut(&key.token_pos)
            )
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_node(&mut self, prev: Option<CacheKey>, node: TraversalNode<R, Q>) -> Option<CacheKey> {
        let key = node.cache_key();
        if let Some(ve) = self.bu.get_mut(&key.root) {
            if let Some(e) = ve.positions.get_mut(&key.token_pos) {
                e.prev.insert(prev.expect("Non first nodes must have prev node!"));
                None
            } else {
                ve.positions.insert(key.token_pos, BUPositionCache::new(prev, node));
                Some(key)
            }
        } else {
            let mut positions: HashMap<_, _> = Default::default();
            positions.insert(key.token_pos, BUPositionCache::new(prev, node));
            self.bu.insert(key.root, BUVertexCache {
                positions
            });
            Some(key)
        }
    }
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
    pub fn on_bu_mismatch(&mut self, key: CacheKey) -> Option<Vec<TraversalNode<R, Q>>> {
        self.get_entry_mut(key).and_then(|e| {
            e.state = CacheState::Mismatch;
            e.waiting
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