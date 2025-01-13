use crate::{
    direction::r#match::MatchDirection,
    graph::{
        getters::ErrorReason,
        kind::GraphKind,
        vertex::{
            child::Child,
            pattern::{
                IntoPattern,
                Pattern,
            },
            wide::Wide,
        },
    },
    path::{
        accessors::role::Start,
        mutators::move_path::key::TokenPosition,
        structs::rooted_path::{
            RootedRangePath,
            SubPath,
        },
    },
    traversal::{
        fold::ErrorState, result::kind::RoleChildPath, traversable::Traversable
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryState {
    pub path: RootedRangePath<Pattern>,
    pub pos: TokenPosition,
}

impl QueryState {
    pub fn new<G: GraphKind, P: IntoPattern>(query: P) -> Result<Self, ErrorState> {
        let entry = G::Direction::head_index(&query.borrow());
        let query = query.into_pattern();
        let first = *query.first().unwrap();
        let len = query.len();
        let pos = first.width().into();
        let query = Self {
            path: RootedRangePath {
                root: query,
                start: SubPath::new(entry).into(),
                end: SubPath::new(entry).into(),
            },
            pos,
        };
        match len {
            0 => Err((ErrorReason::EmptyPatterns, query)),
            1 => Err((ErrorReason::SingleIndex(first), query)),
            _ => Ok(query),
        }
        .map_err(|(reason, _)| ErrorState {
            reason,
            found: None,
        })
    }
    pub fn start_index<Trav: Traversable>(
        &self,
        trav: Trav,
    ) -> Child {
        self.path.role_leaf_child::<Start, _>(&trav)
    }
}
