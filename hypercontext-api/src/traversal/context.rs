use derive_new::new;

use crate::traversal::{
    cache::{
        state::query::QueryState,
        TraversalCache,
    },
    iterator::TraversalIterator,
};
use crate::graph::vertex::pattern::Pattern;

#[derive(Debug, new)]
pub struct QueryStateContext<'c> {
    pub ctx: &'c QueryContext,
    pub state: &'c mut QueryState,
}

#[derive(Debug, new)]
pub struct QueryContext {
    pub query_root: Pattern,
}

#[derive(Debug)]
pub struct TraversalStateContext<'a, 'b: 'a, I: TraversalIterator<'b>> {
    pub query: &'a QueryContext,
    pub cache: &'a mut TraversalCache,
    pub iter: &'a mut I,
    _ty: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b: 'a, I: TraversalIterator<'b>> TraversalStateContext<'a, 'b, I> {
    pub fn new(
        query: &'a QueryContext,
        cache: &'a mut TraversalCache,
        iter: &'a mut I,
    ) -> Self {
        Self {
            query,
            cache,
            iter,
            _ty: Default::default(),
        }
    }
}

impl<'a, 'b: 'a, I: TraversalIterator<'b>> TraversalStateContext<'a, 'b, I> {
    pub fn query_state(
        &self,
        state: &'a mut QueryState,
    ) -> QueryStateContext<'a> {
        QueryStateContext {
            ctx: self.query,
            state,
        }
    }
}
