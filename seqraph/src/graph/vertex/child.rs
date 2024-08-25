use std::{
    borrow::Borrow,
    fmt::Debug,
};

use derive_more::From;
use serde::{
    Deserialize,
    Serialize,
};

use crate::traversal::cache::key::{
    DownKey,
    DownPosition,
    UpKey,
    UpPosition,
};
use crate::graph::vertex::{
    has_vertex_index::HasVertexIndex,
    location::{
        child::ChildLocation,
        pattern::PatternLocation,
        SubLocation,
    },
    PatternId,
    token::NewTokenIndex,
    TokenPosition,
    VertexIndex,
    wide::{
        Wide,
        WideMut,
    },
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, From, Serialize, Deserialize)]
pub struct ChildWidth(pub usize);

impl Borrow<ChildWidth> for Child {
    fn borrow(&self) -> &ChildWidth {
        &self.width
    }
}

#[derive(Debug, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Child {
    pub index: VertexIndex, // the child index
    pub width: ChildWidth,  // the token width
}

impl Child {
    pub fn new(
        index: impl HasVertexIndex,
        width: TokenPosition,
    ) -> Self {
        Self {
            index: index.vertex_index(),
            width: ChildWidth(width),
        }
    }
    pub fn get_width(&self) -> TokenPosition {
        self.width.0
    }
    pub fn get_index(&self) -> VertexIndex {
        self.index
    }
    pub fn to_pattern_location(
        self,
        pattern_id: PatternId,
    ) -> PatternLocation {
        PatternLocation::new(self, pattern_id)
    }
    pub fn to_child_location(
        self,
        sub: SubLocation,
    ) -> ChildLocation {
        ChildLocation::new(self, sub.pattern_id, sub.sub_index)
    }
    pub fn down_key(
        self,
        pos: impl Into<DownPosition>,
    ) -> DownKey {
        DownKey::new(self, pos.into())
    }
    pub fn up_key(
        self,
        pos: impl Into<UpPosition>,
    ) -> UpKey {
        UpKey::new(self, pos.into())
    }
}

impl PartialOrd for Child {
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
    fn hash<H: std::hash::Hasher>(
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
        Self::new(o.vertex_index(), 1)
    }
}

impl IntoIterator for Child {
    type Item = Self;
    type IntoIter = std::iter::Once<Child>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl HasVertexIndex for Child {
    fn vertex_index(&self) -> VertexIndex {
        self.index
    }
}

impl Wide for Child {
    fn width(&self) -> usize {
        self.width.0
    }
}

impl WideMut for Child {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width.0
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
