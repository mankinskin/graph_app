use crate::{
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::IntoPatternLocation,
        },
        wide::Wide,
    },
    impl_cursor_pos,
    path::{
        accessors::{
            child::{
                PathChild,
                RootChildPos,
            },
            role::PathRole,
        },
        mutators::{
            adapters::IntoPrimer,
            move_path::key::TokenPosition,
        },
        structs::{
            query_range_path::FoldablePath,
            role_path::RolePath,
            rooted::{
                pattern_range::PatternRangePath,
                role_path::RootedRolePath,
                root::IndexRoot,
            },
            sub_path::SubPath,
        },
    },
    traversal::traversable::Traversable,
};

use super::bottom_up::parent::ParentState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathCursor<P: FoldablePath> {
    pub path: P,
    /// position relative to start of path
    pub relative_pos: TokenPosition,
}
impl<R: PathRole, P: FoldablePath + PathChild<R>> PathChild<R> for PathCursor<P> {
    fn path_child_location(&self) -> Option<ChildLocation> {
        self.path.path_child_location()
    }
    fn path_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Option<Child> {
        self.path.path_child(trav)
    }
}
impl<R: PathRole, P: RootChildPos<R> + FoldablePath> RootChildPos<R> for PathCursor<P> {
    fn root_child_pos(&self) -> usize {
        RootChildPos::<R>::root_child_pos(&self.path)
    }
}

pub type PatternRangeCursor = PathCursor<PatternRangePath>;
impl_cursor_pos! {
    CursorPosition for PatternRangeCursor, self => self.relative_pos
}

impl IntoPrimer for (Child, PatternRangeCursor) {
    fn into_primer<Trav: Traversable>(
        self,
        _trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState {
        let (c, cursor) = self;
        let width = c.width().into();
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
            cursor,
        }
    }
}
pub trait ToCursor: FoldablePath {
    fn to_cursor(self) -> PathCursor<Self>;
}
impl<P: FoldablePath> ToCursor for P {
    fn to_cursor(self) -> PathCursor<Self> {
        PathCursor {
            path: self,
            relative_pos: TokenPosition::default(),
        }
    }
}
//impl PatternRangeCursor {
//pub fn new<G: GraphKind, P: IntoPattern>(query: P) -> Result<Self, ErrorState> {
//    let entry = G::Direction::head_index(&query.borrow());
//    let query = query.into_pattern();
//    let first = *query.first().unwrap();
//    let len = query.len();
//    //let pos = first.width().into();
//    let query = Self {
//        path: RootedRangePath {
//            root: query,
//            start: SubPath::new(entry).into(),
//            end: SubPath::new(entry).into(),
//        },
//        //pos,
//    };
//    match len {
//        0 => Err((ErrorReason::EmptyPatterns, query)),
//        1 => Err((ErrorReason::SingleIndex(first), query)),
//        _ => Ok(query),
//    }
//    .map_err(|(reason, _)| ErrorState {
//        reason,
//        found: None,
//    })
//}
//}
