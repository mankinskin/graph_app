use super::*;
type StateDepth = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionCache<R: ResultKind> {
    pub top_down: HashMap<CacheKey, ChildLocation>,
    pub bottom_up: HashMap<CacheKey, SubLocation>,
    pub index: Child,
    pub waiting: Vec<(StateDepth, WaitingState<R>)>,
    pub _ty: std::marker::PhantomData<R>,
}
impl<R: ResultKind> PositionCache<R> {
    pub fn start(index: Child) -> Self {
        Self {
            index,
            top_down: Default::default(),
            bottom_up: Default::default(),
            waiting: Default::default(),
            _ty: Default::default(),
        }
    }
    pub fn new<
        Trav: Traversable,
    >(
        trav: &Trav,
        state: &TraversalState<R>,
    ) -> Self {
        //let cache_node = CacheNode::new(node);
        let mut top_down = HashMap::default();
        let mut bottom_up = HashMap::default();
        if let (prev, Some(entry)) = (state.prev_key(), state.entry_location()) {
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
            index: state.target_key(trav).index,
            waiting: Default::default(),
            _ty: Default::default(),
        };
        s
    }
    pub fn add_waiting(&mut self, depth: StateDepth, state: WaitingState<R>) {
        self.waiting.push((depth, state));
    }
    pub fn num_parents(&self) -> usize {
        self.top_down.len()
    }
}
///// Bottom-Up Cache Entry
//#[derive(Clone, Debug, PartialEq, Eq)]
//pub struct VertexCache<R: ResultKind> {
//    pub(crate) positions: HashMap<usize, PositionCache<R>>
//}
//impl<R: ResultKind> VertexCache<R> {
//    pub(crate)fn new_position(
//        &mut self,
//        key: CacheKey,
//        state: &TraversalState<R>,
//    ) {
//        let cache = PositionCache::new(
//            state,
//        );
//        self.positions.insert(
//            key.token_pos,
//            cache,
//        );
//    }
//}