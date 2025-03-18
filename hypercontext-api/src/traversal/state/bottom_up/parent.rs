use crate::{
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
            role::Start,
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
        structs::rooted::role_path::IndexStartPath,
    },
    traversal::{
        cache::key::{
            directed::{
                up::UpKey,
                DirectedKey,
            },
            prev::{
                PrevKey,
                ToPrev,
            },
            props::{
                RootKey,
                TargetKey,
            },
        },
        iterator::policy::DirectedTraversalPolicy,
        state::{
            next_states::{
                NextStates,
                StateNext,
            },
            top_down::{
                child::{
                    ChildState,
                    PathPairMode,
                },
                end::{
                    EndReason,
                    EndState,
                },
            },
            BaseState,
        },
        traversable::{
            TravDir,
            Traversable,
        },
        TraversalKind,
    },
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
};
pub type ParentState = BaseState<IndexStartPath>;

impl TargetKey for ParentState {
    fn target_key(&self) -> DirectedKey {
        self.root_key().into()
    }
}
impl RootKey for ParentState {
    fn root_key(&self) -> UpKey {
        UpKey::new(self.path.root_parent(), self.root_pos.into())
    }
}
impl_cursor_pos! {
    CursorPosition for ParentState, self => self.cursor.relative_pos
}

impl IntoAdvanced for (PrevKey, ParentState) {
    fn into_advanced<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Result<ChildState, Self> {
        let ps = &self.1;
        let entry = ps.path.root_child_location();
        let graph = trav.graph();
        let pattern = ps.path.root_pattern::<Trav>(&graph).clone();
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), entry.sub_index) {
            let root_parent = ps.clone();
            let (
                root_prev,
                ParentState {
                    path,
                    prev_pos,
                    root_pos,
                    cursor,
                },
            ) = self;
            let index = pattern[next];
            Ok(ChildState {
                base: BaseState {
                    prev_pos,
                    root_pos,
                    path: path.into_range(next),
                    cursor,
                },
                mode: PathPairMode::GraphMajor,
                target: DirectedKey::down(index, root_pos),
                root_parent,
                root_prev,
            })
        } else {
            Err(self)
        }
    }
}

impl Ord for ParentState {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.path.root_parent().cmp(&other.path.root_parent())
    }
}
impl PathRaise for ParentState {
    fn path_raise<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) {
        let path = &mut self.path.role_path.sub_path;
        let root = self.path.root.location.to_child_location(path.root_entry);
        path.root_entry = parent_entry.sub_index;
        self.path.root.location = parent_entry.into_pattern_location();
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(root);
        self.prev_pos = self.root_pos;
        self.root_pos
            .advance_key(pattern_width(&pattern[root.sub_index + 1..]));
        if !path.is_empty()
            || TravDir::<Trav>::pattern_index_prev(pattern, root.sub_index).is_some()
        {
            path.path.push(root);
        }
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
impl ParentState {
    pub fn parent_next_states<'a, K: TraversalKind>(
        self,
        trav: &K::Trav,
        //new: Vec<NewEntry>,
        prev: PrevKey,
    ) -> NextStates {
        let key = self.target_key();
        match (prev, self).into_advanced(trav) {
            // first child state in this parent
            Ok(advanced) => {
                let delta = <_ as GraphRootChild<Start>>::root_post_ctx_width(&advanced.path, trav);
                NextStates::Child(StateNext {
                    prev: key.flipped().to_prev(delta),
                    //new,
                    inner: advanced,
                })
            }
            // no child state, bottom up path at end of parent
            Err((_, state)) => state.next_parents::<K>(trav), //, new),
        }
    }
    pub fn next_parents<'a, K: TraversalKind>(
        self,
        trav: &K::Trav,
        //new: Vec<NewEntry>,
    ) -> NextStates {
        // get next parents
        let key = self.target_key();
        let parents = K::Policy::next_parents(trav, &self);
        let delta = self.path.root_post_ctx_width(trav);
        if parents.is_empty() {
            NextStates::End(StateNext {
                prev: key.to_prev(delta),
                //new,
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: self.root_pos,
                    kind: self.path.simplify_to_end(trav),
                    cursor: self.cursor,
                },
            })
        } else {
            NextStates::Parents(StateNext {
                prev: key.to_prev(delta),
                //new,
                inner: parents,
            })
        }
    }
}
