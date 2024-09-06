use crate::graph::vertex::data::VertexData;
use crate::graph::vertex::key::VertexKey;

pub trait HasVertexKey: Sized {
    fn vertex_key(&self) -> VertexKey;
}

impl<I: HasVertexKey> HasVertexKey for &'_ I {
    fn vertex_key(&self) -> VertexKey {
        (**self).vertex_key()
    }
}

impl<I: HasVertexKey> HasVertexKey for &'_ mut I {
    fn vertex_key(&self) -> VertexKey {
        (**self).vertex_key()
    }
}

impl HasVertexKey for VertexKey {
    fn vertex_key(&self) -> VertexKey {
        *self
    }
}

impl HasVertexKey for VertexData {
    fn vertex_key(&self) -> VertexKey {
        self.key
    }
}
