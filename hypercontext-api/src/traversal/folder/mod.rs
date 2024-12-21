use crate::{
    graph::{
        getters::NoMatch,
        vertex::{
            child::Child,
            pattern::{
                IntoPattern, Pattern
            },
        },
    },
    path::{accessors::role::End, structs::query_range_path::QueryRangePath},
    traversal::{
        cache::{
            key::{
                root::RootKey, DirectedKey
            },
            state::{
                end::EndKind,
                query::QueryState,
            },
            TraversalCache,
        },
        context::QueryContext,
        folder::state::{
            FinalState, FoldResult, FoldState
        },
        iterator::TraversalIterator,
        result::TraversalResult,
        traversable::Traversable,
    },
};
use std::borrow::Borrow;
use super::{cache::{key::UpKey, state::{end::EndState, start::StartState}}, context::TraversalStateContext, iterator::traverser::extend::ExtendStates, result::kind::RoleChildPath};

pub mod state;

pub struct FoldFinished {
    end_state: EndState,
    cache: TraversalCache,
    start_index: Child,
    query_root: Pattern,
}
impl FoldFinished {
    pub fn to_traversal_result(self) -> TraversalResult {
        let final_state = FinalState {
            num_parents: self.cache
                .get(&DirectedKey::from(self.end_state.root_key()))
                .unwrap()
                .num_parents(),
            state: &self.end_state,
        };
        let query = final_state.state.query.clone();
        let found_path = if let EndKind::Complete(c) = &final_state.state.kind {
            FoldResult::Complete(*c)
        } else {
            // todo: complete bottom edges of root if
            // assert same root
            //let min_end = end_states.iter()
            //    .min_by(|a, b| a.root_key().index.width().cmp(&b.root_key().index.width()))
            //    .unwrap();
            let root = self.end_state.root_key().index;
            let state = FoldState {
                cache: self.cache,
                root,
                end_state: self.end_state,
                start: self.start_index,
            };
            FoldResult::Incomplete(state)
        };
        TraversalResult {
            query: query.to_rooted(self.query_root),
            result: found_path,
        }
    }
}

pub trait TraversalFolder: Sized + Traversable {
    type Iterator<'a>: TraversalIterator<'a, Trav = Self> + From<&'a Self>
    where
        Self: 'a;

    //#[instrument(skip(self))]
    fn fold_pattern<P: IntoPattern>(
        &self,
        query_pattern: P,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)>
    {
        let query_pattern = query_pattern.into_pattern();

        // build cursor path
        let query = QueryState::new::<Self::Kind, _>(query_pattern.borrow())
            .map_err(|(err, q)| (err, q.to_rooted(query_pattern.clone())))?;

        let query_range_path = query
            .clone()
            .to_rooted(query_pattern.clone());

        let query_root = QueryContext::new(query_pattern);
        self.fold_query(query_root, query_range_path, query)
    }
    //#[instrument(skip(self))]
    fn fold_query(
        &self,
        query_root: QueryContext,
        query_range_path: QueryRangePath,
        query: QueryState,
    ) -> Result<TraversalResult, (NoMatch, QueryRangePath)>
    {

        let start_index = query_range_path.role_leaf_child::<End, _>(self);
        //let query_ctx = QueryContext::new(query_pattern.clone());

        let mut start = StartState {
            index: start_index,
            key: UpKey::new(
                start_index,
                0.into(),
            ),
            query,
        };

        let mut cache = TraversalCache::new(self, start_index);

        let mut states = Self::Iterator::from(self);

        let init = {
            let mut ctx = TraversalStateContext::new(&query_root, &mut cache, &mut states);
            start
                .next_states(&mut ctx)
                .into_states()
                .into_iter()
                .map(|n| (1, n))
        };
        states.extend(init);

        states.fold_states()
    }
}
