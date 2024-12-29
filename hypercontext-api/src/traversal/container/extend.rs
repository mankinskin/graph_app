use crate::traversal::state::traversal::TraversalState;

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
