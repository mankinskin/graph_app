use super::*;

pub mod node;
pub use node::*;
pub mod key;
pub use key::*;

type HashMap<K, V> = DeterministicHashMap<K, V>;

//#[derive(Clone, Debug)]
//pub struct ParentCache {
//    prev: CacheKey,
//    loc: SubLocation,
//}
//#[derive(Clone, Debug)]
//pub struct ChildCache {
//    prev: CacheKey,
//    loc: ChildLocation,
//}
//#[derive(Clone, Debug)]
//pub enum CacheRole {
//    Parent(ParentCache),
//    Start,
//    Child(ChildCache),
//    End(CacheKey),
//}
//#[derive(Clone, Debug)]
//pub struct CacheNode {
//    index: Child,
//    role: CacheRole
//}
//impl CacheNode {
//    pub fn new<
//        R: ResultKind,
//        Q: BaseQuery,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(
//        trav: &Trav,
//        prev: CacheKey,
//        node: TraversalNode<R, Q>,
//    ) -> Self {
//        let (index, role) = match node {
//            TraversalNode::Parent(prev, node) => {
//                let entry = node.path.role_child_location::<Start>();
//                (entry.parent.index(), CacheRole::Parent(ParentCache {
//                    prev,
//                    loc: entry.into_sub_location(),
//                }))
//            },
//            TraversalNode::Child(prev, node) => {
//                let path = node.paths.get_path();
//                (path.role_path_child::<End, _, _>(trav).index(), CacheRole::Child(ChildCache {
//                    prev,
//                    loc: path.role_child_location::<End>(),
//                }))
//            },
//            TraversalNode::Mismatch(prev, found) |
//            TraversalNode::QueryEnd(prev, found) => 
//                (found.path.root_parent().index(), CacheRole::End(prev)),
//            TraversalNode::Start(index, _) => (index, CacheRole::Start),
//        };
//        CacheNode {
//            index: 
//            role,
//        }
//    }
//}
#[derive(Clone, Debug)]
pub struct PositionCache<R: ResultKind, Q: BaseQuery> {
    pub top_down: HashMap<CacheKey, ChildLocation>,
    pub bottom_up: HashMap<CacheKey, SubLocation>,
    pub index: Child,
    //pub query: Q,
    pub waiting: Vec<TraversalNode<R, Q>>,
    _ty: std::marker::PhantomData<R>,
}
impl<R: ResultKind, Q: BaseQuery> PositionCache<R, Q> {
    pub fn new(
        node: &TraversalNode<R, Q>,
    ) -> Self {
        //let cache_node = CacheNode::new(node);
        let mut top_down = HashMap::default();
        let mut bottom_up = HashMap::default();
        if let (Some(prev), Some(entry)) = (node.prev_key(), node.entry_location()) {
            match node.node_direction() {
                NodeDirection::TopDown => {
                    top_down.insert(prev, entry);
                },
                NodeDirection::BottomUp => {
                    bottom_up.insert(prev, entry.into_sub_location());
                },
            }
        }
        let s = Self {
            top_down,
            bottom_up,
            //query: node.query,
            index: node.root_parent(),
            waiting: Default::default(),
            _ty: Default::default(),
        };
        //s.waiting.push(cache_node);
        s
    }
    //pub fn add(&mut self, prev: CacheKey, node: TraversalNode<R, Q>) {
    //    self.waiting.push(CacheNode::new(prev, node));
    //}
}
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct VertexCache<R: ResultKind, Q: BaseQuery> {
    positions: HashMap<usize, PositionCache<R, Q>>
}
impl<R: ResultKind, Q: BaseQuery> VertexCache<R, Q> {
    pub fn on_new_position(
        &mut self,
        key: CacheKey,
        node: &TraversalNode<R, Q>,
    ) {
        let cache = PositionCache::new(
            node,
        );
        self.positions.insert(
            key.token_pos,
            cache,
        );
    }
}
#[derive(Clone, Debug)]
pub struct TraversalCache<R: ResultKind, Q: BaseQuery> {
    entries: HashMap<usize, VertexCache<R, Q>>,
}
impl<R: ResultKind, Q: BaseQuery> TraversalCache<R, Q> {
    pub fn new(start: &StartNode<R, Q>) -> Self {
        Self {
            entries: std::iter::once(
                (
                    start.index.index(),
                    VertexCache {
                        positions: std::iter::once(
                            (
                                0,
                                PositionCache {
                                    index: start.index,
                                    top_down: Default::default(),
                                    bottom_up: Default::default(),
                                    waiting: Default::default(),
                                    _ty: Default::default(),
                                },
                            )
                        ).into_iter().collect::<HashMap<_, _>>()
                    }
                ),
            ).into_iter().collect::<HashMap<_, _>>(),
        }
    }
    pub fn get_entry_mut(&mut self, key: CacheKey) -> Option<&mut PositionCache<R, Q>> {
        self.entries.get_mut(&key.root)
            .and_then(|e|
                e.positions.get_mut(&key.token_pos)
            )
    }
    /// adds node to cache and returns the state of the insertion
    pub fn add_node(&mut self, node: &TraversalNode<R, Q>) -> Result<CacheKey, CacheKey> {
        let key = node.cache_key();
        if let Some(ve) = self.entries.get_mut(&key.root) {
            if let Some(e) = ve.positions.get_mut(&key.token_pos) {
                //waiting
                //e.add(
                //    prev,
                //    node,
                //);
                Err(key)
            } else {
                ve.on_new_position(
                    key,
                    node,
                );
                Ok(key)
            }
        } else {
            self.on_new_vertex(
                key, 
                node,
            );
            Ok(key)
        }
    }
    pub fn on_new_vertex(
        &mut self,
        key: CacheKey,
        node: &TraversalNode<R, Q>,
    ) {
        let mut ve = VertexCache {
            positions: Default::default()
        };
        ve.on_new_position(
            key,
            node,
        );
        self.entries.insert(key.root, ve);
    }
}