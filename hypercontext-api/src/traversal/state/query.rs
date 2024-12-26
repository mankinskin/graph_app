use crate::{
    direction::r#match::MatchDirection, graph::{getters::NoMatch, kind::GraphKind, vertex::child::Child}, path::{
        accessors::role::{
            End,
            Start,
        },
        mutators::move_path::key::TokenPosition,
        structs::{
            role_path::RolePath,
            rooted_path::{RootedPath, RootedRangePath, SubPath},
        },
    }, traversal::{
        result::kind::RoleChildPath, traversable::Traversable
    }
};
use crate::graph::vertex::{
    pattern::{
        IntoPattern,
        Pattern,
    },
    wide::Wide,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryState {
    pub path: RootedRangePath<Pattern>,
    pub pos: TokenPosition,
}

impl QueryState {
    pub fn new<G: GraphKind, P: IntoPattern>(query: P) -> Result<Self, (NoMatch, Self)> {
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
            0 => Err((NoMatch::EmptyPatterns, query)),
            1 => Err((NoMatch::SingleIndex(first), query)),
            _ => Ok(query),
        }
    }
    pub fn start_index<Trav: Traversable>(&self, trav: Trav) -> Child {
        self.path.role_leaf_child::<Start, _>(&trav)
    }
}
