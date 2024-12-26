use itertools::Itertools;

use crate::traversal::{
    cache::key::root::RootKey,state::traversal::TraversalState, traversable::Traversable
};

use super::{pruning::PruningState, StateContainer};
pub trait ExtendStates {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: T,
    );
}


//impl<'a, K: TraversalKind> ExtendStates for TraversalContext<'a, K> {
//    fn extend<
//        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
//        In: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
//    >(
//        &mut self,
//        iter: In,
//    ) {
//        self.ctx.extend(iter)
//    }
//}
