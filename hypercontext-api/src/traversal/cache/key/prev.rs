use crate::traversal::cache::key::DirectedKey;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct PrevKey {
    pub prev_target: DirectedKey,
    pub delta: usize,
}

impl PrevKey {
    pub fn advanced(&self) -> DirectedKey {
        let mut target = self.prev_target;
        target.pos += self.delta;
        target
    }
}

pub trait ToPrev {
    fn to_prev(
        self,
        delta: usize,
    ) -> PrevKey;
}

impl<T: Into<DirectedKey>> ToPrev for T {
    fn to_prev(
        self,
        delta: usize,
    ) -> PrevKey {
        PrevKey {
            prev_target: self.into(),
            delta,
        }
    }
}