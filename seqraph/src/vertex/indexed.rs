use crate::{
    graph::kind::GraphKind,
    traversal::traversable::Traversable,
    vertex::{
        child::Child,
        pattern::Pattern,
        PatternId,
        token::Tokenize,
        VertexIndex,
    },
};

pub trait Indexed: Sized {
    fn vertex_index(&self) -> VertexIndex;
    fn expect_child_patterns<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> crate::vertex::ChildPatterns {
        trav.graph().expect_child_patterns(self).clone()
    }
    fn expect_child_pattern<Trav: Traversable>(
        &self,
        trav: &Trav,
        pid: PatternId,
    ) -> Pattern {
        trav.graph().expect_child_pattern(self, pid).clone()
    }
}

impl<I: Indexed> Indexed for &'_ I {
    fn vertex_index(&self) -> VertexIndex {
        (**self).vertex_index()
    }
}

impl<I: Indexed> Indexed for &'_ mut I {
    fn vertex_index(&self) -> VertexIndex {
        (**self).vertex_index()
    }
}

impl Indexed for VertexIndex {
    fn vertex_index(&self) -> VertexIndex {
        *self
    }
}

impl<G: GraphKind> Indexed for crate::vertex::VertexData<G> {
    fn vertex_index(&self) -> VertexIndex {
        self.index
    }
}

pub trait AsChild: Indexed + crate::vertex::wide::Wide + std::fmt::Debug {
    fn as_child(&self) -> Child {
        Child::new(self.vertex_index(), self.width())
    }
}

impl<T: Indexed + crate::vertex::wide::Wide + std::fmt::Debug> AsChild for T {}

pub trait ToChild: AsChild + Sized + std::fmt::Debug {
    fn to_child(self) -> Child {
        self.as_child()
    }
}

impl<T: AsChild> ToChild for T {}

pub trait MaybeIndexed<T: Tokenize> {
    type Inner: Indexed;
    fn into_inner(self) -> Result<Self::Inner, T>;
}

impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for Result<I, T> {
    type Inner = I;
    fn into_inner(self) -> Result<Self::Inner, T> {
        self
    }
}
//impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for I {
//    type Inner = I;
//    fn into_inner(self) -> Result<Self::Inner, T> {
//        Ok(self)
//    }
//}
