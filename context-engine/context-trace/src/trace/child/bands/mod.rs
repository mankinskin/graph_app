use crate::{
    direction::{
        Direction,
        pattern::PatternDirection,
    },
    graph::{
        kind::DirectionOf,
        vertex::{
            child::Child,
            has_vertex_index::ToChild,
            location::{
                child::ChildLocation,
                pattern::PatternLocation,
            },
        },
    },
    path::{
        mutators::append::PathAppend,
        structs::rooted::role_path::{
            IndexStartPath,
            RootedRolePath,
        },
    },
    trace::has_graph::HasGraph,
};
use itertools::{
    FoldWhile,
    Itertools,
};
use policy::{
    BandExpandingPolicy,
    PostfixExpandingPolicy,
    PrefixExpandingPolicy,
};
use std::collections::VecDeque;

pub mod policy;

pub trait BandIterator<'a, G: HasGraph + 'a>:
    Iterator<Item = (ChildLocation, Child)>
{
    type Policy: BandExpandingPolicy<G>;
    fn band_iter(
        trav: G,
        root: Child,
    ) -> Self;
    fn trav(&self) -> &G;
    fn trav_mut(&mut self) -> &mut G;

    /// get all postfixes of index with their locations
    fn next_children(
        &self,
        index: Child,
    ) -> Vec<(ChildLocation, Child)> {
        Self::Policy::map_batch(
            self.trav().graph().expect_child_patterns(index).iter().map(
                |(pid, pattern)| {
                    Self::Policy::map_band(
                        PatternLocation::new(index, *pid),
                        &pattern,
                    )
                },
            ),
        )
    }
}
use crate::trace::has_graph::TravDir;
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

    /// Calculate the prefix path from this child to the given advanced path
    fn prefix_path<G>(
        &self,
        trav: &G,
        prefix: Child,
    ) -> IndexStartPath
    where
        G: HasGraph + Clone,
    {
        // Find prefix from advanced path in expansion index
        let mut prefix_iter = self.prefix_iter(trav.clone());
        let entry = prefix_iter.next().unwrap().0;

        prefix_iter
            .fold_while(
                RootedRolePath::new(entry),
                |mut acc, (prefix_location, pre)| {
                    acc.path_append(prefix_location);
                    if pre == prefix {
                        FoldWhile::Done(acc)
                    } else {
                        FoldWhile::Continue(acc)
                    }
                },
            )
            .into_inner()
            .into()
    }
}
impl<T: ToChild> HasChildRoleIters for T {}

pub type PostfixIterator<'a, G> = BandExpandingIterator<
    'a,
    G,
    PostfixExpandingPolicy<DirectionOf<<G as HasGraph>::Kind>>,
>;

pub type PrefixIterator<'a, G> = BandExpandingIterator<
    'a,
    G,
    PrefixExpandingPolicy<DirectionOf<<G as HasGraph>::Kind>>,
>;

#[derive(Debug)]
pub struct BandExpandingIterator<'a, G, P>
where
    G: HasGraph + 'a,
    P: BandExpandingPolicy<G>,
{
    trav: G,
    queue: VecDeque<(ChildLocation, Child)>,
    last: (Option<ChildLocation>, Child),
    _ty: std::marker::PhantomData<&'a P>,
}

impl<'a, G, P> BandIterator<'a, G> for BandExpandingIterator<'a, G, P>
where
    G: HasGraph + 'a,
    P: BandExpandingPolicy<G>,
{
    type Policy = P;
    fn band_iter(
        trav: G,
        root: Child,
    ) -> Self {
        Self {
            trav,
            queue: VecDeque::new(),
            last: (None, root),
            _ty: Default::default(),
        }
    }
    fn trav(&self) -> &G {
        &self.trav
    }
    fn trav_mut(&mut self) -> &mut G {
        &mut self.trav
    }
}

impl<'a, G, P> Iterator for BandExpandingIterator<'a, G, P>
where
    G: HasGraph,
    P: BandExpandingPolicy<G>,
{
    type Item = (ChildLocation, Child);

    fn next(&mut self) -> Option<Self::Item> {
        //let mut segment = None;
        let next = self.next_children(self.last.1);
        if self.queue.is_empty() {
            self.queue.extend(next);
        }
        self.queue.pop_front().map(|(location, node)| {
            self.last.0 = Some(location.clone());
            self.last.1 = node;
            (location, node)
        })
    }
}
