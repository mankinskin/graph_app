use super::*;

#[derive(Clone, Debug)]
pub struct PositionCache<R: ResultKind, Q: BaseQuery> {
    pub top_down: HashMap<CacheKey, ChildLocation>,
    pub bottom_up: HashMap<CacheKey, SubLocation>,
    pub index: Child,
    pub waiting: Vec<(usize, TraversalState<R, Q>)>,
    _ty: std::marker::PhantomData<R>,
}
impl<R: ResultKind, Q: BaseQuery> PositionCache<R, Q> {
    pub fn start(index: Child) -> Self {
        Self {
            index,
            top_down: Default::default(),
            bottom_up: Default::default(),
            waiting: Default::default(),
            _ty: Default::default(),
        }
    }
    pub fn new(
        state: &TraversalState<R, Q>,
    ) -> Self {
        //let cache_node = CacheNode::new(node);
        let mut top_down = HashMap::default();
        let mut bottom_up = HashMap::default();
        if let (Some(prev), Some(entry)) = (state.prev_key(), state.entry_location()) {
            match state.node_direction() {
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
            index: state.root_parent(),
            waiting: Default::default(),
            _ty: Default::default(),
        };
        s
    }
    pub fn add_waiting(&mut self, depth: usize, state: TraversalState<R, Q>) {
        self.waiting.push((depth, state));
    }
}
/// Bottom-Up Cache Entry
#[derive(Clone, Debug)]
pub struct VertexCache<R: ResultKind, Q: BaseQuery> {
    pub(crate) positions: HashMap<usize, PositionCache<R, Q>>
}
impl<R: ResultKind, Q: BaseQuery> VertexCache<R, Q> {
    pub(crate)fn new_position(
        &mut self,
        key: CacheKey,
        state: &TraversalState<R, Q>,
    ) {
        let cache = PositionCache::new(
            state,
        );
        self.positions.insert(
            key.token_pos,
            cache,
        );
    }
}