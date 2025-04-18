pub mod batch;

use batch::ParentBatch;

use crate::traversal::{
    iterator::policy::DirectedTraversalPolicy,
    state::{
        child::{
            ChildState,
            RootChildState,
        },
        end::{
            EndKind,
            EndReason,
            EndState,
        },
        BaseState,
        StateNext,
    },
    TraversalKind,
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
        structs::rooted::{
            role_path::IndexStartPath,
            root::RootedPath,
        },
        RoleChildPath,
    },
    trace::{
        cache::{
            entry::new::NewParent,
            key::{
                directed::{
                    up::UpKey,
                    DirectedKey,
                },
                prev::ToPrev,
                props::{
                    RootKey,
                    TargetKey,
                },
            },
        },
        traversable::{
            TravDir,
            Traversable,
        },
    },
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
};
#[derive(Clone, Debug)]
pub enum BUNext {
    Parents(StateNext<ParentBatch>),
    End(StateNext<EndState>),
}
pub type ParentState = BaseState<IndexStartPath>;

pub trait IntoPrimer: Sized {
    fn into_primer<Trav: Traversable>(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState;
}

impl From<ParentState> for NewParent {
    fn from(state: ParentState) -> Self {
        Self {
            root: state.root_key(),
            entry: state.path.role_root_child_location(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParentNext {
    BU(BUNext),
    Child(StateNext<RootChildState>),
}
impl ParentState {
    pub fn parent_next_states<'a, K: TraversalKind>(
        self,
        trav: &K::Trav,
        //prev: PrevKey,
    ) -> ParentNext {
        let key = self.target_key();
        match self.into_advanced(trav) {
            // first child state in this parent
            Ok(advanced) => {
                let delta = <_ as GraphRootChild<Start>>::root_post_ctx_width(
                    &advanced.path,
                    trav,
                );
                ParentNext::Child(StateNext {
                    prev: key.flipped().to_prev(delta),
                    inner: advanced,
                })
            },
            // no child state, bottom up path at end of parent
            Err(state) => ParentNext::BU(state.next_parents::<K>(trav)),
        }
    }
    pub fn next_parents<'a, K: TraversalKind>(
        self,
        trav: &K::Trav,
    ) -> BUNext {
        // get next parents
        let key = self.target_key();

        let delta = self.path.root_post_ctx_width(trav);
        if let Some(batch) = K::Policy::next_batch(trav, &self) {
            BUNext::Parents(StateNext {
                prev: key.to_prev(delta),
                inner: batch,
            })
        } else {
            BUNext::End(StateNext {
                prev: key.to_prev(delta),
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: self.root_pos,
                    kind: EndKind::simplify_path(self.path, trav),
                    cursor: self.cursor,
                },
            })
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

impl IntoAdvanced for ParentState {
    type Next = RootChildState;
    fn into_advanced<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Result<Self::Next, Self> {
        let entry = self.path.root_child_location();
        let graph = trav.graph();
        let pattern = self.path.root_pattern::<Trav>(&graph).clone();

        if let Some(next) = TravDir::<Trav>::pattern_index_next(
            pattern.borrow(),
            entry.sub_index,
        ) {
            let root_parent = self.clone();
            let ParentState {
                path,
                prev_pos,
                root_pos,
                cursor,
            } = self;
            let index = pattern[next];
            Ok(RootChildState {
                child: ChildState {
                    base: BaseState {
                        prev_pos,
                        root_pos,
                        path: path.into_range(next),
                        cursor,
                    },
                    target: DirectedKey::down(index, root_pos),
                },
                root_parent,
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
            || TravDir::<Trav>::pattern_index_prev(pattern, root.sub_index)
                .is_some()
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
