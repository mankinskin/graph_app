#[macro_use]
#[cfg(test)]
pub mod vkey {
    use crate::traversal::traversable::{
        Traversable,
        TravToken,
    };
    use std::{
        borrow::Borrow,
        fmt::Display,
        hash::{
            Hash,
            Hasher,
        },
    };

    use crate::graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    };


    pub type VertexCacheKey = LabelledKey;

    pub fn labelled_key<Trav: Traversable>(
        trav: &Trav,
        child: Child,
    ) -> VertexCacheKey
    where
        TravToken<Trav>: Display,
    {
        LabelledKey::build(trav, child)
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LabelledKey {
        index: crate::graph::vertex::VertexIndex,
        label: String,
    }

    impl LabelledKey {
        pub fn new(
            child: impl Borrow<Child>,
            label: impl ToString,
        ) -> Self {
            Self {
                label: label.to_string(),
                index: child.borrow().vertex_index(),
            }
        }
        pub fn build<Trav: Traversable>(
            trav: &Trav,
            child: Child,
        ) -> Self
        where
            TravToken<Trav>: Display,
        {
            let index = child.vertex_index();
            Self {
                label: trav.graph().index_string(index),
                index,
            }
        }
    }

    impl Borrow<crate::graph::vertex::VertexIndex> for LabelledKey {
        fn borrow(&self) -> &crate::graph::vertex::VertexIndex {
            &self.index
        }
    }

    impl Hash for LabelledKey {
        fn hash<H: Hasher>(
            &self,
            h: &mut H,
        ) {
            self.index.hash(h)
        }
    }

    impl Display for LabelledKey {
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }
    #[macro_export]
    macro_rules! lab {
        ($x:ident) => {
            crate::traversal::cache::labelled_key::vkey::LabelledKey::new($x, stringify!($x))
        };
    }
    pub use lab;
}

#[cfg(not(test))]
pub mod vkey {
    use crate::traversal::traversable::{
        Traversable,
        TravToken,
    };
    use std::fmt::Display;
    use crate::graph::vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
    };

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
}
