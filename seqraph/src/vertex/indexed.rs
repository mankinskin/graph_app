use super::*;

pub trait Vertexed: Indexed {
    fn vertex<'g, T: Tokenize>(
        &'g self,
        graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        graph.expect_vertex_data(self.index())
    }
}
impl Vertexed for VertexIndex {}
impl Vertexed for Child {}
impl<V: Vertexed> Vertexed for &'_ V {
    fn vertex<'g, T: Tokenize>(
        &'g self,
        graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        (**self).vertex(graph)
    }
}
impl<V: Vertexed> Vertexed for &'_ mut V {
    fn vertex<'g, T: Tokenize>(
        &'g self,
        graph: &'g Hypergraph<T>,
    ) -> &'g VertexData {
        (**self).vertex(graph)
    }
}
pub trait Indexed {
    fn index(&self) -> &VertexIndex;
}
impl<I: Indexed> Indexed for &'_ I {
    fn index(&self) -> &VertexIndex {
        (**self).index()
    }
}
impl<I: Indexed> Indexed for &'_ mut I {
    fn index(&self) -> &VertexIndex {
        (**self).index()
    }
}
impl Indexed for VertexIndex {
    fn index(&self) -> &VertexIndex {
        self
    }
}

pub trait AsChild: Indexed + Wide {
    fn as_child(&self) -> Child {
        Child::new(self.index(), self.width())
    }
}
impl<T: Indexed + Wide> AsChild for T {}

pub trait ToChild: AsChild + Sized {
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
