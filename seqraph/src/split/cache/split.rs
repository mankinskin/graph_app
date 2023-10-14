use crate::*;

pub trait SplitInner: Debug + Clone {}
impl<T: Debug + Clone> SplitInner for T {}

#[derive(Debug, Clone)]
pub struct Split<T: SplitInner = Child> {
    pub left: T,
    pub right: T,
}
impl<T: SplitInner> Split<T> {
    pub fn new(left: T, right: T) -> Self {
        Self {
            left,
            right,
        }
    }
}
impl<I, T: SplitInner + Extend<I> + IntoIterator<Item=I>> Split<T> {
    pub fn infix(&mut self, mut inner: Split<T>) {
        self.left.extend(inner.left);
        inner.right.extend(self.right.clone());
        self.right = inner.right;
    }
}
pub type FinalSplit = Split<Child>;
