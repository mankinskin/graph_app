use crate::traversal::{
    container::ExtendStates,
    state::traversal::TraversalState,
};

use super::StateContainer;

#[derive(Debug, Default)]
pub struct DftStack {
    stack: Vec<(usize, TraversalState)>,
}

//impl From<StartState> for DftStack {
//    fn from(start: StartState) -> Self {
//        Self {
//            stack: Vec::from([(0, TraversalState::Start(start))]),
//            _ty: Default::default(),
//        }
//    }
//}
impl StateContainer for DftStack {
    fn clear(&mut self) {
        self.stack.clear()
    }
}
impl FromIterator<(usize, TraversalState)> for DftStack {
    fn from_iter<T: IntoIterator<Item = (usize, TraversalState)>>(
        iter: T
    ) -> Self {
        Self {
            stack: FromIterator::from_iter(iter),
        }
    }
}

impl Iterator for DftStack {
    type Item = (usize, TraversalState);
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

impl ExtendStates for DftStack {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: T,
    ) {
        self.stack.extend(iter.into_iter().rev())
    }
}
