use super::*;
use std::{
    fmt::Debug,
    hash::Hasher,
    borrow::Borrow,
};

#[derive(Debug, Eq, Clone, Copy)]
pub struct Child {
    pub index: VertexIndex,   // the child index
    pub width: TokenPosition, // the token width
}
impl Child {
    pub fn new(
        index: impl Indexed,
        width: TokenPosition,
    ) -> Self {
        Self {
            index: index.index(),
            width,
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width
    }
    pub fn get_index(&self) -> VertexIndex {
        self.index
    }
    pub fn to_pattern_location(self, pattern_id: PatternId) -> PatternLocation {
        PatternLocation::new(self, pattern_id)
    }
    pub fn to_child_location(self, sub: SubLocation) -> ChildLocation {
        ChildLocation::new(self, sub.pattern_id, sub.sub_index)
    }
    pub fn top_down(self, pos: impl Into<TokenLocation>) -> DirectedKey {
        DirectedKey::down(self, pos)
    }
    pub fn bottom_up(self, pos: impl Into<TokenLocation>) -> DirectedKey {
        DirectedKey::up(self, pos)
    }
}
impl std::cmp::PartialOrd for Child {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl<A: Borrow<Child>, B: Borrow<Child>> From<Result<A, B>> for Child {
    fn from(value: Result<A, B>) -> Self {
        match value {
            Ok(a) => *a.borrow(),
            Err(b) => *b.borrow(),
        }
    }
}
impl std::hash::Hash for Child {
    fn hash<H: Hasher>(
        &self,
        h: &mut H,
    ) {
        self.index.hash(h);
    }
}
//impl std::cmp::Ord for Child {
//    fn cmp(
//        &self,
//        other: &Self,
//    ) -> std::cmp::Ordering {
//        self.index.cmp(&other.index)
//    }
//}
impl PartialEq for Child {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.index == other.index
    }
}
impl PartialEq<VertexIndex> for Child {
    fn eq(
        &self,
        other: &VertexIndex,
    ) -> bool {
        self.index == *other
    }
}
impl PartialEq<VertexIndex> for &'_ Child {
    fn eq(
        &self,
        other: &VertexIndex,
    ) -> bool {
        self.index == *other
    }
}
impl PartialEq<VertexIndex> for &'_ mut Child {
    fn eq(
        &self,
        other: &VertexIndex,
    ) -> bool {
        self.index == *other
    }
}
impl<T: Into<Child> + Clone> From<&'_ T> for Child {
    fn from(o: &'_ T) -> Self {
        (*o).clone().into()
    }
}
impl From<NewTokenIndex> for Child {
    fn from(o: NewTokenIndex) -> Self {
        Self::new(o.index(), 1)
    }
}
impl IntoIterator for Child {
    type Item = Self;
    type IntoIter = std::iter::Once<Child>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl Indexed for Child {
    fn index(&self) -> VertexIndex {
        self.index
    }
}
impl Wide for Child {
    fn width(&self) -> usize {
        self.width
    }
}
impl WideMut for Child {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
    }
}
impl Borrow<[Child]> for Child {
    fn borrow(&self) -> &[Child] {
        std::slice::from_ref(self)
    }
}
impl AsRef<[Child]> for Child {
    fn as_ref(&self) -> &[Child] {
        self.borrow()
    }
}