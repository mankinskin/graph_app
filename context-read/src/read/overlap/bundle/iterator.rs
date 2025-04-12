use async_std::path::Iter;

use super::Bundle;

pub struct BundleIterator {}

impl Iterator for BundleIterator {
    type Item = Bundle;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
