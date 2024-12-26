use derive_new::new;

use crate::traversal::traversable::Traversable;
use crate::traversal::{
    cache::TraversalCache,
    state::query::QueryState,
    iterator::TraversalIterator,
};
use crate::graph::vertex::pattern::Pattern;

use super::{TraversalContext, TraversalKind};

//#[derive(Debug, new)]
//pub struct QueryStateContext<'c> {
//    pub ctx: &'c QueryContext,
//    pub state: &'c mut QueryState,
//}

//#[derive(Debug, new)]
//pub struct QueryContext {
//    pub query_root: Pattern,
//}

//#[derive(Debug)]
//pub struct TraversalStateContext<'a, 'b: 'a, K: TraversalKind> {
//    pub ctx: &'a mut TraversalContext<'b, K>,
//}
//
//impl<'a, 'b: 'a, K: TraversalKind> TraversalStateContext<'a, 'b, K> {
//    pub fn new(
//        ctx: &'a mut TraversalContext<'b, K>,
//    ) -> Self {
//        Self {
//            ctx,
//        }
//    }
//}
//
//impl<K: TraversalKind> TraversalStateContext<'_, '_, K>
//{
//    pub fn trav(&self) -> &'_ K::Trav {
//        self.ctx.trav
//    }
//}