use crate::*;

pub mod key;
pub use key::*;
pub mod entry;
pub use entry::*;
pub mod state;
pub use state::*;
pub mod trace;

#[cfg(test)]
mod vkey {
    use super::*;
    pub type VertexCacheKey = LabelledKey;
    pub fn build_key<Trav: Traversable>(trav: &Trav, child: Child) -> VertexCacheKey
        where TravToken<Trav>: Display
    {
        LabelledKey::build(trav, child)
    }
    macro_rules! lab {
        ($x:ident) => {
            LabelledKey::new($x, stringify!($x))
        }
    }
    pub(crate) use lab;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LabelledKey {
        index: VertexIndex,
        label: String,
    }
    impl LabelledKey {
        pub fn new(child: impl Borrow<Child>, label: impl ToString) -> Self {
            Self {
                label: label.to_string(),            
                index: child.borrow().vertex_index(),
            }
        }
        pub fn build<Trav: Traversable>(trav: &Trav, child: Child) -> Self
            where TravToken<Trav>: Display
        {
            let index = child.vertex_index();
            Self {
                label: trav.graph().index_string(index),            
                index,
            }
        }
    }
    impl Borrow<VertexIndex> for LabelledKey {
        fn borrow(&self) -> &VertexIndex {
            &self.index
        }
    }
    impl Hash for LabelledKey {
        fn hash<H: Hasher>(&self, h: &mut H) {
            self.index.hash(h)
        }
    }
    impl Display for LabelledKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }
}

#[cfg(not(test))]
mod vkey {
    use super::*;
    pub type VertexCacheKey = VertexIndex;
    pub fn build_key<Trav: Traversable>(trav: &Trav, child: Child) -> VertexCacheKey
        where TravToken<Trav>: Display
    {
        child.vertex_index()
    }
}
pub use vkey::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraversalCache {
    pub(crate) entries: HashMap<VertexCacheKey, VertexCache>,
}

impl TraversalCache
{
    pub fn new<Trav: Traversable>(
        trav: &Trav,
        start_index: Child,
        query_state: QueryState,
    ) -> (StartState, Self)
        where TravToken<Trav>: Display
    {
        let mut entries = HashMap::default();

        entries.insert(build_key(trav, start_index), VertexCache::start(start_index));
        let key = UpKey::new(
            start_index,
            TokenLocation(start_index.width()).into(),
        );
        let start = StartState {
            index: start_index,
            key,
            query: query_state,
        };
        (start, Self {
            entries,
        })
    }
    pub fn add_state<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        state: NewEntry,
        add_edges: bool,
    ) -> (DirectedKey, bool)
        where TravToken<Trav>: Display
    {
        let key = state.target_key();
        if let Some(ve) = self.entries.get_mut(&key.index.vertex_index()) {
            if let Some(_) = ve.get_mut(&key.pos) {
                (key, false)
            } else {
                drop(ve);

                let prev = add_edges.then(||
                    self.force_mut(
                        trav,
                        &state.prev_key(),
                    )
                );
                let pe = PositionCache::new(
                    prev,
                    key,
                    state,
                );
                let ve = self.entries.get_mut(&key.index.vertex_index()).unwrap();
                ve.insert(
                    &key.pos,
                    pe,
                );
                (key, true)
            }
        } else {
            self.new_vertex(
                trav,
                key, 
                state,
                add_edges,
            );
            (key, true)
        }
    }
    pub fn trace_path<
        Trav: Traversable,
        P: RoleChildPath + GraphRootChild<End> + HasRolePath<End>,
    >(
        &mut self,
        trav: &Trav,
        root_entry: usize,
        path: &P,
        root_up_pos: TokenLocation,
        add_edges: bool,
    )
        where TravToken<Trav>: Display
    {
        let graph = trav.graph();
        let root_exit = path.role_root_child_location::<End>();

        if add_edges && path.raw_child_path::<End>().is_empty() && graph.expect_is_at_end(&root_exit) {
            return;
        }
        let root_up_key = UpKey::new(
            path.root_parent(),
            root_up_pos.into(),
        );
        let pattern = graph.expect_pattern_at(root_exit);

        // path width
        let root_down_pos = root_up_pos + pattern.get(root_entry+1..root_exit.sub_index)
            .map(|p| pattern_width(p)).unwrap_or_default();

        let root_down_key = DownKey::new(
            path.root_parent(),
            root_down_pos.into(),
        );
        let exit_down_key = DownKey::new(
            graph.expect_child_at(&root_exit),
            root_down_pos.into(),
        );
        self.add_state(
            trav,
            NewEntry {
                prev: root_down_key.into(),
                root_pos: root_up_pos,
                kind: NewKind::Child(NewChild {
                    root: root_up_key,
                    target: exit_down_key.into(),
                    end_leaf: Some(root_exit),
                }),
            },
            add_edges,
        );
        let mut prev_key: DirectedKey = root_down_key.into();
        for loc in path.raw_child_path::<End>() {
            (prev_key, _) = self.add_state(
                trav,
                NewEntry {
                    prev: prev_key,
                    root_pos: root_up_pos,
                    kind: NewKind::Child(NewChild {
                        root: root_up_key.into(),
                        target: DirectedKey::down(
                            graph.expect_child_at(loc),
                            root_down_pos.0 + graph.expect_child_offset(loc),
                        ),
                        end_leaf: None,
                    })
                },
                add_edges,
            );
        }
    }
    fn new_vertex<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        key: DirectedKey,
        state: NewEntry,
        add_edges: bool,
    )
        where TravToken<Trav>: Display
    {
        let mut ve = VertexCache::from(key.index);
        let prev = add_edges.then(||
            self.force_mut(
                trav,
                &state.prev_key()
            )
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
        self.entries.insert(build_key(trav, key.index), ve);
    }
    pub fn continue_waiting(
        &mut self,
        key: &UpKey,
    ) -> Vec<(usize, TraversalState)> {
        self.get_mut(&(DirectedKey::from(*key)))
            .unwrap()
            .waiting
            .drain(..)
            .map(|(d, s)| (d, s.into()))
            .collect()
    }
    pub fn force_mut<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        key: &DirectedKey,
    ) -> &mut PositionCache
        where TravToken<Trav>: Display
    {
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
                self.entries.insert(build_key(trav, key.index), ve);
            }
        }
        self.expect_mut(key)
    }
    pub fn get_vertex(&self, key: &Child) -> Option<&VertexCache> {
        self.entries.get(&key.index.vertex_index())
    }
    pub fn get_vertex_mut(&mut self, key: &Child) -> Option<&mut VertexCache> {
        self.entries.get_mut(&key.index.vertex_index())
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
    pub fn exists_vertex(&self, key: &Child) -> bool {
        if let Some(_) = self.entries.get(&key.vertex_index()) {
            true
        } else {
            false
        }
    }
    pub fn exists(&self, key: &DirectedKey) -> bool {
        if let Some(ve) = self.entries.get(&key.index.vertex_index()) {
            ve.get(&key.pos).is_some()
        } else {
            false
        }
    }
}