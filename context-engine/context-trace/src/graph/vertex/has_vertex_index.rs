use std::fmt::Debug;
use crate::graph::vertex::{
    child::Child,
    data::VertexData,
    VertexIndex,
    wide::Wide,
};

pub trait HasVertexIndex: Sized {
    fn vertex_index(&self) -> VertexIndex;
    //fn expect_child_patterns<Trav: HasGraph>(
    //    &self,
    //    trav: &Trav,
    //) -> ChildPatterns {
    //    trav.graph().expect_child_patterns(self).clone()
    //}
    //fn expect_child_pattern<Trav: HasGraph>(
    //    &self,
    //    trav: &Trav,
    //    pid: PatternId,
    //) -> Pattern {
    //    trav.graph().expect_child_pattern(self, pid).clone()
    //}
}

impl<I: HasVertexIndex> HasVertexIndex for &'_ I {
    fn vertex_index(&self) -> VertexIndex {
        (**self).vertex_index()
    }
}

impl<I: HasVertexIndex> HasVertexIndex for &'_ mut I {
    fn vertex_index(&self) -> VertexIndex {
        (**self).vertex_index()
    }
}

impl HasVertexIndex for VertexIndex {
    fn vertex_index(&self) -> VertexIndex {
        *self
    }
}

impl HasVertexIndex for VertexData {
    fn vertex_index(&self) -> VertexIndex {
        self.index
    }
}
impl HasVertexIndex for Child {
    fn vertex_index(&self) -> VertexIndex {
        self.index
    }
}

pub trait ToChild: HasVertexIndex + Wide + Debug {
    fn to_child(&self) -> Child {
        Child::new(self.vertex_index(), self.width())
    }
}

impl<T: HasVertexIndex + Wide + Debug> ToChild for T {}

//pub trait MaybeIndexed<T: Tokenize> {
//    type Inner: HasVertexIndex;
//    fn into_inner(self) -> Result<Self::Inner, T>;
//}
//
//impl<I: HasVertexIndex, T: Tokenize> MaybeIndexed<T> for Result<I, T> {
//    type Inner = I;
//    fn into_inner(self) -> Result<Self::Inner, T> {
//        self
//    }
//}
//impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for I {
//    type Inner = I;
//    fn into_inner(self) -> Result<Self::Inner, T> {
//        Ok(self)
//    }
//}
