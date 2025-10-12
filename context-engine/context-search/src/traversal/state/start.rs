use crate::traversal::cursor::{PatternCursor};
use crate::{
    fold::{
        foldable::ErrorState,
        result::FinishedKind,
    },
    r#match::iterator::CompareParentBatch,
    traversal::{
        policy::DirectedTraversalPolicy,
        TraversalKind,
    },
};
use context_trace::*;

#[derive(Debug, PartialEq, Eq)]
pub struct StartCtx<K: TraversalKind> {
    pub index: Child,
    pub cursor: PatternCursor,
    pub trav: K::Trav,
}

impl<K: TraversalKind> HasVertexIndex for StartCtx<K> {
    fn vertex_index(&self) -> VertexIndex {
        self.index.vertex_index()
    }
}
impl<K: TraversalKind> Wide for StartCtx<K> {
    fn width(&self) -> usize {
        self.index.width()
    }
}
impl<K: TraversalKind> StartCtx<K> {
    pub fn get_parent_batch(&self) -> Result<CompareParentBatch, ErrorState> {
        let mut cursor = self.cursor.clone();
        if cursor.advance(&self.trav).is_continue() {
            let batch = K::Policy::gen_parent_batch(
                &self.trav,
                self.index,
                |trav, p| self.index.into_primer(trav, p),
            );

            Ok(CompareParentBatch { batch, cursor })
        } else {
            Err(ErrorState {
                reason: ErrorReason::SingleIndex(Box::new(IndexWithPath {
                    index: self.index,
                    path: self.cursor.path.clone().into(),
                })),
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
impl IntoPrimer for Child {
    fn into_primer<G: HasGraph>(
        self,
        _trav: &G,
        parent_entry: ChildLocation,
    ) -> ParentState {
        let width = self.width().into();
        ParentState {
            prev_pos: width,
            root_pos: width,
            path: RootedRolePath {
                root: IndexRoot {
                    location: parent_entry.into_pattern_location(),
                },
                role_path: RolePath {
                    sub_path: SubPath {
                        root_entry: parent_entry.sub_index,
                        path: vec![],
                    },
                    _ty: Default::default(),
                },
            },
        }
    }
}

//impl RootKey for StartState {
//    fn root_key(&self) -> UpKey {
//        UpKey::new(self.index, TokenPosition(self.index.width()).into())
//    }
//}
