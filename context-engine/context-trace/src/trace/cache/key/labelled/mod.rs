#[cfg(not(any(test, feature = "test-api")))]
pub mod normal;
#[cfg(not(any(test, feature = "test-api")))]
pub use normal::*;

#[cfg(any(test, feature = "test-api"))]
pub mod test;

#[cfg(any(test, feature = "test-api"))]
pub use test::*;

use crate::{
    graph::vertex::has_vertex_index::HasVertexIndex,
    trace::has_graph::HasGraph,
};
pub trait Labelling: HasVertexIndex {
    fn labelled<G: HasGraph>(
        self,
        trav: &'_ G,
    ) -> Labelled<Self>;
}
impl<T: HasVertexIndex> Labelling for T {
    fn labelled<G: HasGraph>(
        self,
        trav: &'_ G,
    ) -> Labelled<Self> {
        #[cfg(any(test, feature = "test-api"))]
        {
            Labelled::build(trav, self)
        }

        #[cfg(not(any(test, feature = "test-api")))]
        {
            self
        }
    }
}
