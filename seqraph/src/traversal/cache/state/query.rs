use crate::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryState {
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
    pub pos: TokenLocation,
}
impl QueryState {
    pub fn to_cached(self, cache: &mut TraversalCache) -> CachedQuery<'_> {
        CachedQuery {
            state: self,
            cache,
        }
    }
    pub fn to_rooted(self, root: Pattern) -> QueryRangePath {
        QueryRangePath {
            root,
            start: self.start,
            end: self.end,
        }
    }
    pub fn new<
        D: MatchDirection,
        P: IntoPattern,
    >(query: P) -> Result<Self, (NoMatch, Self)> {
        let entry = D::head_index(query.borrow());
        let query = query.into_pattern();
        let first = *query.first().unwrap();
        let len = query.len();
        let query = Self::new_range(entry, entry, TokenLocation::default());
        match len {
            0 => Err((NoMatch::EmptyPatterns, query)),
            1 => Err((NoMatch::SingleIndex(first), query)),
            _ => Ok(query)
        }
    }
    fn new_range(entry: usize, exit: usize, pos: TokenLocation) -> Self {
        Self {
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
            pos,
        }
    }
}
pub struct CachedQuery<'c> {
    pub cache: &'c mut TraversalCache, 
    pub state: QueryState,
}