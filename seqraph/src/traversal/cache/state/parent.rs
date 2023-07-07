use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentState {
    pub prev_pos: TokenLocation,
    pub root_pos: TokenLocation,
    pub path: Primer,
    pub query: QueryState,
}

impl Ord for ParentState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.root_parent().cmp(
            &other.path.root_parent()
        )
    }
}
impl PartialOrd for ParentState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl ParentState {
    pub fn next_states<'a, 'b: 'a, I: TraversalIterator<'b>>(
        self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        let key = self.target_key();
        match self.into_advanced(ctx.trav()) {
            // first child state in this parent
            Ok(advanced) => NextStates::Child(
                StateNext {
                    prev: key.flipped(),
                    new,
                    inner: advanced
                },
            ),
            // no child state, bottom up path at end of parent
            Err(state) => state.next_parents(
                ctx,
                new,
            )
        }
    }
    pub fn next_parents<'a, 'b: 'a, I: TraversalIterator<'b>>(
        self,
        ctx: &mut TraversalContext<'a, 'b, I>,
        new: Vec<NewEntry>,
    ) -> NextStates {
        // get next parents
        let key = self.target_key();
        let parents = I::Policy::next_parents(
            ctx.trav(),
            &self,
        );
        if parents.is_empty() {
            NextStates::End(StateNext {
                prev: key,
                new,
                inner: EndState {
                    reason: EndReason::Mismatch,
                    root_pos: self.root_pos,
                    kind: self.path.simplify(ctx.trav()),
                    query: self.query,
                },
            })
        } else {
            NextStates::Parents(StateNext {
                prev: key,
                new,
                inner: parents,
            })
        }
    }
}