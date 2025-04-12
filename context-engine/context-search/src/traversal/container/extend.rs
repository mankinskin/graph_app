use crate::traversal::state::traversal::TraversalState;

pub trait ExtendStates {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        next: T,
    );
}
