use crate::shared::*;

#[allow(unused)]
pub type Dft<'a, Trav, S> = OrderedTraverser<'a, Trav, S, DftStack>;

#[derive(Debug)]
pub struct DftStack
{
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
impl NodeVisitor for DftStack {
    fn clear(&mut self) {
        self.stack.clear()
    }
}
impl Iterator for DftStack
{
    type Item = (usize, TraversalState);
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}
impl ExtendStates for DftStack
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter=It>
    >(&mut self, iter: T) {
        self.stack.extend(iter.into_iter().rev())
    }
}
impl Default for DftStack
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}