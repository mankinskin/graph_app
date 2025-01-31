use crate::path::{
    mutators::move_path::key::TokenPosition,
    structs::{
        query_range_path::FoldablePath,
        rooted::pattern_range::PatternRangePath,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathCursor<P: FoldablePath> {
    pub path: P,
    /// position relative to start of path
    pub relative_pos: TokenPosition,
}

pub type RangeCursor = PathCursor<PatternRangePath>;

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
//impl RangeCursor {
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
