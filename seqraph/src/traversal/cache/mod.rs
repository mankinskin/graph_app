use super::*;

pub mod node;
pub use node::*;
pub mod key;
pub use key::*;

type HashMap<K, V> = DeterministicHashMap<K, V>;

#[derive(Clone, Debug)]
pub struct ParentCache {
    prev: CacheKey,
    loc: SubLocation,
}
#[derive(Clone, Debug)]
pub struct ChildCache {
    prev: CacheKey,
    loc: ChildLocation,
}
#[derive(Clone, Debug)]
pub enum CacheRole {
    Parent(ParentCache),
    Start,
    Child(ChildCache),
    End(CacheKey),
}
#[derive(Clone, Debug)]
pub struct CacheNode {
    index: Child,
    role: CacheRole
}
impl CacheNode {
    pub fn new<R: ResultKind, Q: TraversalQuery>(prev: CacheKey, node: TraversalNode<R, Q>) -> Self {
        let (index, role) = match node {
            TraversalNode::Parent(node) => {
                let entry = node.path.child_location();
                (entry.parent, CacheRole::Parent(ParentCache {
                    prev,
                    loc: entry.into_sub_location(),
                }))
            },
            TraversalNode::Child(node) => {
                let path = node.paths.get_path();
                (path.get_descendant(), CacheRole::Child(ChildCache {
                    prev,
                    loc: path.get_descendant_location(),
                }))
            },
            TraversalNode::Mismatch(found) |
            TraversalNode::QueryEnd(found) => 
                (found.child_location().parent, CacheRole::End(prev)),
            TraversalNode::Start(index, _) => (index, CacheRole::Start),
        };
        CacheNode {
            index: 
            role,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PositionCache<R: ResultKind, Q: TraversalQuery> {
    pub top_down: HashMap<CacheKey, ChildLocation>,
    pub bottom_up: HashMap<CacheKey, SubLocation>,
    pub index: Child,
    pub query: Q,
    pub waiting: Vec<CacheNode>,
    _ty: std::marker::PhantomData<R>,
}
impl<R: ResultKind, Q: TraversalQuery> PositionCache<R, Q> {
    pub fn new(prev: CacheKey, node: TraversalNode<R, Q>) -> Self {
        let cache_node = CacheNode::new(prev, node);
        let s = Self {
            top_down: HashMap::default(),
            bottom_up: HashMap::default(),
            query: node.query,
            index: cache_node.index,
            waiting: Default::default(),
        };
        s.waiting.push(cache_node);
        s
    }
    pub fn add(&mut self, prev: CacheKey, node: TraversalNode<R, Q>) {
        self.waiting.push(CacheNode::new(prev, node));
    }
}
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct VertexCache<R: ResultKind, Q: TraversalQuery> {
    positions: HashMap<usize, PositionCache<R, Q>>
}
#[derive(Clone, Debug)]
pub struct TraversalCache<R: ResultKind, Q: TraversalQuery> {
    entries: HashMap<usize, VertexCache<R, Q>>,
}
impl<R: ResultKind, Q: TraversalQuery> TraversalCache<R, Q> {
    pub fn new(index: usize, query: Q) -> (Self, CacheKey) {
        let s = Self {
            entries: std::iter::once(
                (
                    index,
                    VertexCache {
                        positions: std::iter::once(
                            (
                                0,
                                PositionCache {
                                    start: Some((index, query)),
                                    td: Default::default(),
                                    bu: Default::default(),
                                }
                            )
                        ).into_iter().collect::<HashMap<_, _>>()
                    }
                ),
            ).into_iter().collect::<HashMap<_, _>>(),
        };
        (s, CacheKey::new(index, 0))
    }
    pub fn get_entry_mut(&mut self, key: CacheKey) -> Option<&mut PositionCache<R, Q>> {
        self.entries.get_mut(&key.root)
            .and_then(|e|
                e.positions.get_mut(&key.token_pos)
            )
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_node(&mut self, prev: CacheKey, node: TraversalNode<R, Q>) -> Option<CacheKey> {
        let key = node.cache_key();
        if let Some(ve) = self.entries.get_mut(&key.root) {
            if let Some(e) = ve.positions.get_mut(&key.token_pos) {
                e.add(
                    prev,
                    node,
                );
                None
            } else {
                self.on_new_position(
                    ve,
                    key,
                    prev,
                    node,
                );
                Some(key)
            }
        } else {
            self.on_new_vertex(
                key, 
                prev,
                node,
            );
            Some(key)
        }
    }
    pub fn on_new_position(
        &mut self,
        ve: &mut VertexCache<R, Q>,
        key: CacheKey,
        prev: CacheKey,
        node: TraversalNode<R, Q>,
    ) {
        let cache = PositionCache::new(
            prev,
            node,
        );
        ve.positions.insert(
            key.token_pos,
            cache,
        );
    }
    pub fn on_new_vertex(
        &mut self,
        key: CacheKey,
        prev: CacheKey,
        node: TraversalNode<R, Q>,
    ) {
        let mut ve = VertexCache {
            positions: Default::default()
        };
        self.on_new_position(
            &mut ve,
            key,
            prev,
            node,
        );
        self.entries.insert(key.root, ve);
    }
}