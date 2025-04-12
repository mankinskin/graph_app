#[macro_use]
#[cfg(not(any(test, feature = "test-api")))]
pub mod vkey {
    use crate::{
        graph::vertex::{
            child::Child,
            has_vertex_index::HasVertexIndex,
        },
        traversal::traversable::{
            TravToken,
            Traversable,
        },
    };
    use std::fmt::Display;

    pub type VertexCacheKey = crate::graph::vertex::VertexIndex;

    pub fn labelled_key<Trav: Traversable>(
        _trav: &Trav,
        child: Child,
    ) -> VertexCacheKey
    where
        TravToken<Trav>: Display,
    {
        child.vertex_index()
    }

    #[macro_export]
    macro_rules! lab {
        ($x:ident) => {
            $x.vertex_index()
        };
    }
}

#[cfg(any(test, feature = "test-api"))]
pub mod vkey {
    pub use crate::tests::label_key::*;
}

pub use vkey::*;
