use crate::traversal::state::{
    child::{
        ChildState,
        RootChildState,
    },
    BaseState,
};
use context_trace::{
    direction::pattern::PatternDirection,
    graph::vertex::{
        location::{
            child::ChildLocation,
            pattern::IntoPatternLocation,
        },
        pattern::pattern_width,
    },
    impl_cursor_pos,
    path::{
        accessors::{
            child::root::GraphRootChild,
            root::{
                GraphRoot,
                RootPattern,
            },
        },
        mutators::{
            adapters::IntoAdvanced,
            move_path::key::AdvanceKey,
            raise::PathRaise,
        },
        structs::rooted::{
            role_path::IndexStartPath,
            root::RootedPath,
        },
    },
    trace::{
        cache::key::{
            directed::{
                down::DownKey,
                up::UpKey,
                DirectedKey,
            },
            props::{
                RootKey,
                TargetKey,
            },
        },
        has_graph::{
            HasGraph,
            TravDir,
        },
    },
};
use derive_more::{
    Deref,
    DerefMut,
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::VecDeque,
};

#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct ParentBatch {
    pub parents: VecDeque<ParentState>,
}
pub type ParentState = BaseState<IndexStartPath>;

impl IntoAdvanced for ParentState {
    type Next = RootChildState;
    fn into_advanced<G: HasGraph>(
        self,
        trav: &G,
    ) -> Result<Self::Next, Self> {
        let entry = self.path.root_child_location();
        let graph = trav.graph();
        let pattern = self.path.root_pattern::<G>(&graph).clone();
        if let Some(next_i) =
            TravDir::<G>::pattern_index_next(pattern.borrow(), entry.sub_index)
        {
            let root_parent = self.clone();
            let ParentState {
                path,
                prev_pos,
                root_pos,
                cursor,
            } = self;
            let index = pattern[next_i];
            //println!("{:#?}", (&pattern, entry, index));
            Ok(RootChildState {
                child: ChildState {
                    base: BaseState {
                        prev_pos,
                        root_pos,
                        path: path.into_range(next_i),
                        cursor,
                    },
                    target: DownKey::new(index, root_pos.into()),
                },
                root_parent,
            })
        } else {
            Err(self)
        }
    }
}
impl PathRaise for ParentState {
    fn path_raise<G: HasGraph>(
        &mut self,
        trav: &G,
        parent_entry: ChildLocation,
    ) {
        // new root
        let path = &mut self.path.role_path.sub_path;

        let graph = trav.graph();
        let prev_pattern =
            graph.expect_pattern_at(self.path.root.location.clone());

        self.prev_pos = self.root_pos;
        self.root_pos
            .advance_key(pattern_width(&prev_pattern[path.root_entry + 1..]));

        let prev = self.path.root.location.to_child_location(path.root_entry);
        path.root_entry = parent_entry.sub_index;
        self.path.root.location = parent_entry.into_pattern_location();

        // path raise is only called when path matches until end
        // avoid pointing path to the first child
        if !path.is_empty()
            || TravDir::<G>::pattern_index_prev(prev_pattern, prev.sub_index)
                .is_some()
        {
            path.path.push(prev);
        }
    }
}

impl<P: RootedPath + GraphRoot> TargetKey for BaseState<P> {
    fn target_key(&self) -> DirectedKey {
        self.root_key().into()
    }
}
impl<P: RootedPath + GraphRoot> RootKey for BaseState<P> {
    fn root_key(&self) -> UpKey {
        UpKey::new(self.path.root_parent(), self.root_pos.into())
    }
}
impl_cursor_pos! {
    CursorPosition for ParentState, self => self.cursor.relative_pos
}
impl Ord for ParentState {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.path.root_parent().cmp(&other.path.root_parent())
    }
}
impl PartialOrd for ParentState {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
