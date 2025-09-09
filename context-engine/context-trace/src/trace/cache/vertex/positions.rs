use crate::{
    HashMap,
    path::mutators::move_path::key::TokenPosition,
    trace::cache::position::PositionCache,
};
use derive_more::derive::{
    Deref,
    DerefMut,
    IntoIterator,
};
use std::iter::FromIterator;

#[derive(
    Clone, Debug, PartialEq, Eq, Default, IntoIterator, Deref, DerefMut,
)]
pub struct DirectedPositions {
    entries: HashMap<TokenPosition, PositionCache>,
}
impl FromIterator<(TokenPosition, PositionCache)> for DirectedPositions {
    fn from_iter<T: IntoIterator<Item = (TokenPosition, PositionCache)>>(
        iter: T
    ) -> Self {
        Self {
            entries: FromIterator::from_iter(iter),
        }
    }
}
impl Extend<(TokenPosition, PositionCache)> for DirectedPositions {
    fn extend<T: IntoIterator<Item = (TokenPosition, PositionCache)>>(
        &mut self,
        iter: T,
    ) {
        for (k, v) in iter {
            if let Some(c) = self.entries.get_mut(&k) {
                //assert!(c.index == v.index);
                c.top.extend(v.top);
                c.bottom.extend(v.bottom);
            } else {
                self.entries.insert(k, v);
            }
        }
    }
}
