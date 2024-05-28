#[cfg(test)]
pub mod vkey {
    use crate::traversal::traversable::{
        TravToken,
        Traversable,
    };
    use std::{
        borrow::Borrow,
        fmt::Display,
        hash::{
            Hash,
            Hasher,
        },
    };

    use crate::vertex::{
        child::Child,
        indexed::Indexed,
    };

    macro_rules! lab {
        ($x:ident) => {
            LabelledKey::new($x, stringify!($x))
        };
    }
    pub(crate) use lab;

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
        index: crate::vertex::VertexIndex,
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

    impl Borrow<crate::vertex::VertexIndex> for LabelledKey {
        fn borrow(&self) -> &crate::vertex::VertexIndex {
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
}

#[cfg(not(test))]
pub mod vkey {
    use crate::{
        traversal::traversable::{
            TravToken,
            Traversable,
        },
        vertex::{
            child::Child,
            indexed::Indexed,
        },
    };
    use std::fmt::Display;

    pub type VertexCacheKey = crate::vertex::VertexIndex;

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
