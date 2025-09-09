use crate::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
    },
    path::{
        accessors::{
            child::RootChildIndex,
            role::PathRole,
            root::{
                GraphRootPattern,
                PatternRoot,
            },
        },
        structs::rooted::root::RootedPath,
    },
    trace::has_graph::HasGraph,
};
use auto_impl::auto_impl;

#[auto_impl(&, & mut)]
pub trait RootChild<R>: RootChildIndex<R> {
    fn root_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Child;
}
#[macro_export]
macro_rules! impl_child {
    {
        RootChild for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl<R: PathRole> $crate::path::accessors::child::root::RootChild<R> for $target
            where $target: RootChildIndex<R>
        {
            fn root_child<
                G: HasGraph,
            >(& $self_, $trav: &G) -> $crate::graph::vertex::child::Child {
                $func
            }
        }
    };
}

/// used to get a direct child in a Graph
pub trait GraphRootChild<R: PathRole>: RootedPath + GraphRootPattern {
    fn root_child_location(&self) -> ChildLocation;
    fn graph_root_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Child {
        *trav.graph().expect_child_at(
            <_ as GraphRootChild<R>>::root_child_location(self),
        )
    }
    fn get_outer_width<G: HasGraph>(
        &self,
        trav: &G,
    ) -> usize {
        let i = self.root_child_location().sub_index;
        let g = trav.graph();
        let p = self.graph_root_pattern::<G>(&g);
        R::outer_ctx_width(p, i)
    }
    fn get_inner_width<G: HasGraph>(
        &self,
        trav: &G,
    ) -> usize {
        let i = self.root_child_location().sub_index;
        let g = trav.graph();
        let p = self.graph_root_pattern::<G>(&g);
        R::inner_width(p, i)
    }
}
impl<R: PathRole> GraphRootChild<R> for ChildLocation {
    fn root_child_location(&self) -> ChildLocation {
        self.clone()
    }
}
// used to get a direct child of a pattern
pub trait PatternRootChild<R>: RootChildIndex<R> + PatternRoot {
    fn pattern_root_child(&self) -> Child {
        PatternRoot::pattern_root_pattern(self)[self.root_child_index()]
    }
}
