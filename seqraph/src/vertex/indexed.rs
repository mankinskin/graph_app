use crate::traversal::traversable::Traversable;

use super::*;

pub trait Indexed: Sized {
    fn index(&self) -> VertexIndex;
    fn expect_child_patterns<
        Trav: Traversable,
    >(&self, trav: &Trav) -> ChildPatterns {
        trav.graph().expect_child_patterns(self).clone()
    }
    fn expect_child_pattern<
        Trav: Traversable,
    >(&self, trav: &Trav, pid: PatternId) -> Pattern {
        trav.graph().expect_child_pattern(self, pid).clone()
    }
}
impl<I: Indexed> Indexed for &'_ I {
    fn index(&self) -> VertexIndex {
        (**self).index()
    }
}
impl<I: Indexed> Indexed for &'_ mut I {
    fn index(&self) -> VertexIndex {
        (**self).index()
    }
}
impl Indexed for VertexIndex {
    fn index(&self) -> VertexIndex {
        *self
    }
}
impl Indexed for VertexData {
    fn index(&self) -> VertexIndex {
        self.index
    }
}

pub trait AsChild: Indexed + Wide + std::fmt::Debug {
    fn as_child(&self) -> Child {
        Child::new(self.index(), self.width())
    }
}
impl<T: Indexed + Wide + std::fmt::Debug> AsChild for T {}

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
