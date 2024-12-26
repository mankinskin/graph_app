use std::cmp::Ordering;

use crate::graph::vertex::{location::child::ChildLocation, wide::Wide};


pub trait TraversalOrder: Wide {
    fn sub_index(&self) -> usize;
    fn cmp(
        &self,
        other: impl TraversalOrder,
    ) -> Ordering {
        match other.width().cmp(&self.width()) {
            Ordering::Equal => self.sub_index().cmp(&other.sub_index()),
            r => r,
        }
    }
}

impl<T: TraversalOrder> TraversalOrder for &T {
    fn sub_index(&self) -> usize {
        TraversalOrder::sub_index(*self)
    }
}

impl TraversalOrder for ChildLocation {
    fn sub_index(&self) -> usize {
        self.sub_index
    }
}