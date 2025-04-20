use crate::traversal::{
    fold::foldable::ErrorState,
    iterator::policy::DirectedTraversalPolicy,
    result::FinishedKind,
    state::{
        cursor::PatternRangeCursor,
        ParentState,
    },
    ParentBatch,
    TraversalKind,
};
use context_trace::{
    graph::{
        getters::ErrorReason,
        vertex::{
            child::Child,
            location::{
                child::ChildLocation,
                pattern::IntoPatternLocation,
            },
            wide::Wide,
        },
    },
    path::{
        mutators::move_path::advance::Advance,
        structs::{
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::IndexRoot,
            },
            sub_path::SubPath,
        },
    },
    trace::has_graph::HasGraph,
};
#[derive(Debug, PartialEq, Eq)]
pub struct StartCtx<K: TraversalKind> {
    pub index: Child,
    pub cursor: PatternRangeCursor,
    pub trav: K::Trav,
}

impl<K: TraversalKind> StartCtx<K> {
    pub fn get_parent_batch(&self) -> Result<ParentBatch, ErrorState> {
        let mut cursor = self.cursor.clone();
        if cursor.advance(&self.trav).is_continue() {
            //prev: self.key.to_prev(delta),
            Ok(K::Policy::gen_parent_batch(
                &self.trav,
                self.index,
                |trav, p| (self.index, cursor.clone()).into_primer(trav, p),
            ))
        } else {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(self.index),
                found: Some(FinishedKind::Complete(self.index)),
            })
        }
    }
}
pub trait IntoPrimer: Sized {
    fn into_primer<G: HasGraph>(
        self,
        trav: &G,
        parent_entry: ChildLocation,
    ) -> ParentState;
}
impl IntoPrimer for (Child, PatternRangeCursor) {
    fn into_primer<G: HasGraph>(
        self,
        _trav: &G,
        parent_entry: ChildLocation,
    ) -> ParentState {
        let (c, cursor) = self;
        let width = c.width().into();
        ParentState {
            prev_pos: width,
            root_pos: width,
            path: RootedRolePath {
                root: IndexRoot {
                    location: parent_entry.clone().into_pattern_location(),
                },
                role_path: RolePath {
                    sub_path: SubPath {
                        root_entry: parent_entry.sub_index,
                        path: vec![],
                    },
                    _ty: Default::default(),
                },
            },
            cursor,
        }
    }
}

//impl RootKey for StartState {
//    fn root_key(&self) -> UpKey {
//        UpKey::new(self.index, TokenPosition(self.index.width()).into())
//    }
//}
