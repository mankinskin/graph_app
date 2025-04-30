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
            root::{
                GraphRoot,
                RootPattern,
            },
        },
        mutators::{
            adapters::IntoAdvanced,
            move_path::{
                advance::Advance,
                key::AdvanceKey,
            },
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
use std::{
    borrow::Borrow,
    cmp::Ordering,
};
//#[derive(Clone, Debug)]
//pub enum BUNext {
//    Parents(StateNext<ParentBatch>),
//    End(StateNext<EndState>),
//}
//#[derive(Clone, Debug)]
//pub enum ParentNext {
//    BU(BUNext),
//    Child(StateNext<RootChildState>),
//}
use context_trace::trace::cache::key::directed::down::DownKey;
//impl From<ParentState> for UpEdit {
//    fn from(state: ParentState) -> Self {
//        Self {
//            key: state.root_key(),
//            entry: state.path.role_root_child_location(),
//        }
//    }
//}

pub type ParentState = BaseState<IndexStartPath>;
impl ParentState {
    //pub fn parent_next_states<'a, K: TraversalKind>(
    //    self,
    //    trav: &K::Trav,
    //    //prev: PrevKey,
    //) -> ParentNext {
    //    let key = self.target_key();
    //    match self.into_advanced(trav) {
    //        // first child state in this parent
    //        Ok(advanced) => {
    //            let delta = <_ as GraphRootChild<Start>>::root_post_ctx_width(
    //                &advanced.path,
    //                trav,
    //            );
    //            ParentNext::Child(StateNext {
    //                prev: key.flipped().to_prev(delta),
    //                inner: advanced,
    //            })
    //        },
    //        // no child state, bottom up path at end of parent
    //        Err(state) => ParentNext::BU(state.next_parents::<K>(trav)),
    //    }
    //}
    pub fn next_parents<'a, K: TraversalKind>(
        mut self,
        trav: &K::Trav,
    ) -> Result<(Self, ParentBatch), EndState> {
        // get next parents
        //let key = self.target_key();
        //let delta = self.path.root_post_ctx_width(trav);

        let cursor = self.cursor.clone();
        if self.cursor.advance(trav).is_continue() {
            if let Some(batch) = K::Policy::next_batch(trav, &self) {
                Ok((self, batch))
            } else {
                Err(EndState {
                    reason: EndReason::Mismatch,
                    //root_pos: self.root_pos,
                    kind: EndKind::from_start_path(
                        self.path,
                        self.root_pos,
                        trav,
                    ),
                    cursor,
                })
            }
        } else {
            Err(EndState {
                reason: EndReason::QueryEnd,
                //root_pos: self.root_pos,
                kind: EndKind::from_start_path(self.path, self.root_pos, trav),
                cursor,
            })
        }
    }
}

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
            println!("{:#?}", (&pattern, entry, index));
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

        let prev = self.path.root.location.to_child_location(path.root_entry);

        let graph = trav.graph();
        let prev_pattern = graph.expect_pattern_at(prev.clone());

        self.prev_pos = self.root_pos;
        self.root_pos
            .advance_key(pattern_width(&prev_pattern[prev.sub_index + 1..]));

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
