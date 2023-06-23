use crate::*;

pub mod key;
pub use key::*;
pub mod entry;
pub use entry::*;
pub mod state;
pub use state::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalCache {
    pub(crate) query_root: Pattern,
    pub(crate) entries: HashMap<VertexIndex, VertexCache>,
}
impl TraversalCache {
    pub fn new(start: &StartState, query: Pattern) -> (DirectedKey, Self) {
        let mut entries = HashMap::default();
        entries.insert(start.index.index(), VertexCache::start(start.index));
        (start.root_key(), Self {
            query_root: query,
            entries,
        })
    }
    pub fn get_vertex(&self, key: &Child) -> Option<&VertexCache> {
        self.entries.get(&key.index.index())
    }
    pub fn get_vertex_mut(&mut self, key: &Child) -> Option<&mut VertexCache> {
        self.entries.get_mut(&key.index.index())
    }
    pub fn expect_vertex(&self, key: &Child) -> &VertexCache {
        self.get_vertex(key).unwrap()
    }
    pub fn expect_vertex_mut(&mut self, key: &Child) -> &mut VertexCache {
        self.get_vertex_mut(key).unwrap()
    }
    pub fn get(&self, key: &DirectedKey) -> Option<&PositionCache> {
        self.get_vertex(&key.index)
            .and_then(|ve|
                ve.get(&key.pos)
            )
    }
    pub fn get_mut(&mut self, key: &DirectedKey) -> Option<&mut PositionCache> {
        self.get_vertex_mut(&key.index)
            .and_then(|ve| {
                //println!("get_entry positions {:#?}: {:#?}", key, ve.positions);
                ve.get_mut(&key.pos)
            })
    }
    pub fn expect(&self, key: &DirectedKey) -> &PositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(&mut self, key: &DirectedKey) -> &mut PositionCache {
        self.get_mut(key).unwrap()
    }
    pub fn force_mut(&mut self, key: &DirectedKey) -> &mut PositionCache {
        if !self.exists(&key) {
            let pe = PositionCache::start(key.index);
            if let Some(ve) = self.get_vertex_mut(&key.index) {
                ve.insert(
                    &key.pos,
                    pe,
                );
            } else {
                let mut ve = VertexCache::from(key.index);
                ve.insert(
                    &key.pos,
                    pe,
                );
                self.entries.insert(key.index.index(), ve);
            }
        }
        self.expect_mut(key)
    }
    pub fn exists_vertex(&self, key: &Child) -> bool {
        if let Some(_) = self.entries.get(&key.index()) {
            true
        } else {
            false
        }
    }
    pub fn exists(&self, key: &DirectedKey) -> bool {
        if let Some(ve) = self.entries.get(&key.index.index()) {
            ve.get(&key.pos).is_some()
        } else {
            false
        }
    }
    pub fn add_state(&mut self, state: NewEntry, add_edges: bool) -> (DirectedKey, bool) {
        let key = state.target_key();
        if let Some(ve) = self.entries.get_mut(&key.index.index()) {
            if let Some(_) = ve.get_mut(&key.pos) {
                (key, false)
            } else {
                drop(ve);

                let prev = add_edges.then(||
                    self.force_mut(&state.prev_key())
                );
                let pe = PositionCache::new(
                    prev,
                    key,
                    state
                );
                let ve = self.entries.get_mut(&key.index.index()).unwrap();
                ve.insert(
                    &key.pos,
                    pe,
                );
                (key, true)
            }
        } else {
            self.new_vertex(
                key, 
                state,
                add_edges
            );
            (key, true)
        }
    }
    pub fn add_path<
        Trav: Traversable,
        P: RoleChildPath + GraphRootChild<End> + HasRolePath<End>,
    >(
        &mut self,
        trav: &Trav,
        root_entry: usize,
        path: &P,
        root_pos: TokenLocation,
        add_edges: bool,
    ) {
        let graph = trav.graph();
        let root_exit = path.role_root_child_location::<End>();
        if add_edges && path.raw_child_path::<End>().is_empty() && graph.expect_is_at_end(&root_exit) {
            return;
        }
        let root_up_key = DirectedKey::up(
            path.root_parent(),
            root_pos,
        );
        let pattern = graph.expect_pattern_at(root_exit);
        let root_down_pos = root_pos + pattern.get(root_entry+1..root_exit.sub_index)
            .map(|p| pattern_width(p)).unwrap_or_default();
        let root_key = DirectedKey::down(
            path.root_parent(),
            root_down_pos,
        );
        self.add_state(NewEntry {
                prev: root_key,
                root_pos,
                kind: NewKind::Child(NewChild {
                    root: root_up_key,
                    target: DirectedKey::down(
                        graph.expect_child_at(&root_exit),
                        root_down_pos,
                    ),
                    end_leaf: Some(root_exit),
                }),
            },
            add_edges,
        );
        let mut prev_key = root_key;
        for loc in path.raw_child_path::<End>() {
            (prev_key, _) = self.add_state(NewEntry {
                    prev: prev_key,
                    root_pos,
                    kind: NewKind::Child(NewChild {
                        root: root_key,
                        target: DirectedKey::down(
                            graph.expect_child_at(loc),
                            root_pos.pos + graph.expect_child_offset(loc),
                        ),
                        end_leaf: None,
                    })
                },
                add_edges,
            );
        }
    }
    fn new_vertex(
        &mut self,
        key: DirectedKey,
        state: NewEntry,
        add_edges: bool,
    ) {
        let mut ve = VertexCache::from(key.index);
        let prev = add_edges.then(||
            self.force_mut(&state.prev_key())
        );
        let pe = PositionCache::new(
            prev,
            key,
            state
        );
        ve.insert(
            &key.pos,
            pe,
        );
        self.entries.insert(key.index.index(), ve);
    }
    //pub fn position_bottoms(&self, root: &Child) -> Vec<(TokenLocation, DirectedKey, SubLocation)> {
    //    self.entries.get(&root.index).unwrap()
    //        .positions.iter().map(|(&pos, c)| {
    //            c.edges.bottom.iter().map(move |(key, loc)| (pos, *key, *loc))
    //        })
    //        .flatten()
    //        .collect_vec()
    //}
    //pub fn trace_down_from(
    //    &mut self,
    //    key: &DirectedKey,
    //) {
    //    let mut queue = VecDeque::new();
    //    let mut node = self.expect_entry_mut(key);
    //    queue.extend(node.edges.bottom.iter());
    //    let mut prev = key;
    //    while let Some(node_key) = queue.pop_front() {
    //        let mut node = self.expect_entry_mut(node_key);
    //        node.edges.top.insert(*prev);
    //        queue.extend(node.edges.bottom.iter());
    //    }
    //}
    pub fn continue_waiting(
        &mut self,
        key: &DirectedKey,
    ) -> Vec<(usize, TraversalState)> {
        self.get_mut(key)
            .unwrap()
            .waiting
            .drain(..)
            .map(|(d, s)| (d, s.into()))
            .collect()
    }

}