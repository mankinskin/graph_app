use crate::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryState {
    pub start: RolePath<Start>,
    pub end: RolePath<End>,
    pub pos: TokenLocation,
}
impl QueryState {
    pub fn to_ctx<'a, 'b: 'a, I: TraversalIterator<'b>>(
        &'a mut self,
        ctx: &TraversalContext<'a, 'b, I>,
    ) -> QueryStateContext<'a> {
        ctx.query_state(self)
    }
    pub fn to_rooted(self, root: Pattern) -> QueryRangePath {
        QueryRangePath {
            root,
            start: self.start,
            end: self.end,
        }
    }
    pub fn new<
        G: GraphKind,
        P: IntoPattern,
    >(query: P) -> Result<Self, (NoMatch, Self)> {
        let entry = G::Direction::head_index(query.borrow());
        let query = query.into_pattern();
        let first = *query.first().unwrap();
        let len = query.len();
        let query = Self::new_range(entry, entry, first.width().into());
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