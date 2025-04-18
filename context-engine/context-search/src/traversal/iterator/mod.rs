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
    trace::traversable::{
        TravDir,
        Traversable,
    },
};

pub mod bands;
pub mod policy;

pub trait HasChildRoleIters: ToChild {
    fn postfix_iter<'a, Trav: Traversable + 'a>(
        &self,
        trav: Trav,
    ) -> PostfixIterator<'a, Trav>
    where
        <TravDir<Trav> as Direction>::Opposite: PatternDirection,
    {
        PostfixIterator::band_iter(trav, self.to_child())
    }
    fn prefix_iter<'a, Trav: Traversable + 'a>(
        &self,
        trav: Trav,
    ) -> PrefixIterator<'a, Trav> {
        PrefixIterator::band_iter(trav, self.to_child())
    }
}
