use crate::traversal::iterator::bands::{
    BandIterator,
    PostfixIterator,
    PrefixIterator,
};
use context_trace::{
    direction::{
        pattern::PatternDirection,
        Direction,
    },
    graph::vertex::has_vertex_index::ToChild,
    trace::has_graph::{
        HasGraph,
        TravDir,
    },
};

pub mod bands;
pub mod policy;

pub trait HasChildRoleIters: ToChild {
    fn postfix_iter<'a, G: HasGraph + 'a>(
        &self,
        trav: G,
    ) -> PostfixIterator<'a, G>
    where
        <TravDir<G> as Direction>::Opposite: PatternDirection,
    {
        PostfixIterator::band_iter(trav, self.to_child())
    }
    fn prefix_iter<'a, G: HasGraph + 'a>(
        &self,
        trav: G,
    ) -> PrefixIterator<'a, G> {
        PrefixIterator::band_iter(trav, self.to_child())
    }
}
