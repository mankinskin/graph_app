use crate::{
    graph::vertex::has_vertex_index::HasVertexIndex,
    trace::{
        cache::{
            Child,
            VertexIndex,
        },
        has_graph::{
            HasGraph,
            TravToken,
        },
    },
};
use derive_more::Deref;
use std::{
    borrow::Borrow,
    fmt::Display,
    hash::{
        Hash,
        Hasher,
    },
};

impl<T> Borrow<T> for Labelled<T> {
    fn borrow(&self) -> &T {
        &self.index
    }
}

pub fn labelled<G: HasGraph, T: HasVertexIndex>(
    trav: &G,
    index: T,
) -> Labelled<T>
where
    TravToken<G>: Display,
{
    Labelled::<T>::build(trav, index)
}

#[derive(Clone, Debug, PartialEq, Eq, Deref)]
pub struct Labelled<T> {
    #[deref]
    index: T,
    label: String,
}
impl From<Labelled<Child>> for Labelled<VertexIndex> {
    fn from(c: Labelled<Child>) -> Self {
        Labelled {
            index: c.index.vertex_index(),
            label: c.label,
        }
    }
}
impl<T> Labelled<T> {
    pub fn new(
        index: T,
        label: impl ToString,
    ) -> Self {
        Self {
            label: label.to_string(),
            index,
        }
    }
    pub fn build<G: HasGraph>(
        trav: &G,
        index: T,
    ) -> Self
    where
        TravToken<G>: Display,
        T: HasVertexIndex,
    {
        let vi = index.vertex_index();
        Self {
            label: trav.graph().index_string(vi),
            index,
        }
    }
}

impl<T: Hash> Hash for Labelled<T> {
    fn hash<H: Hasher>(
        &self,
        h: &mut H,
    ) {
        self.index.hash(h)
    }
}

impl<T> Display for Labelled<T> {
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
        $crate::trace::cache::key::labelled::Labelled::new($x, stringify!($x))
    };
}
pub use lab;
