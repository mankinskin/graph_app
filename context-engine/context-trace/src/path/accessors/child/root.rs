use crate::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
        pattern::{
            pattern_post_ctx_width,
            pattern_pre_ctx_width,
        },
    },
    path::accessors::{
        child::RootChildPos,
        root::{
            GraphRootPattern,
            PatternRoot,
        },
    },
    trace::has_graph::HasGraph,
};
use auto_impl::auto_impl;

#[auto_impl(&, & mut)]
pub trait RootChild<R>: RootChildPos<R> {
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
            where $target: RootChildPos<R>
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
pub trait GraphRootChild<R>: GraphRootPattern + RootChildPos<R> {
    fn root_child_location(&self) -> ChildLocation;
    fn graph_root_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Child {
        trav.graph()
            .expect_child_at(<_ as GraphRootChild<R>>::root_child_location(
                self,
            ))
            .clone()
    }
    fn root_post_ctx_width<G: HasGraph>(
        &self,
        trav: &G,
    ) -> usize {
        let i = self.root_child_location().sub_index;
        let g = trav.graph();
        let p = self.graph_root_pattern::<G>(&g);
        pattern_post_ctx_width(p, i)
    }
    fn root_pre_ctx_width<G: HasGraph>(
        &self,
        trav: &G,
    ) -> usize {
        let i = self.root_child_location().sub_index;
        let g = trav.graph();
        let p = self.graph_root_pattern::<G>(&g);
        pattern_pre_ctx_width(p, i)
    }
}

/// used to get a direct child of a pattern
pub trait PatternRootChild<R>: RootChildPos<R> + PatternRoot {
    fn pattern_root_child(&self) -> Child {
        PatternRoot::pattern_root_pattern(self)[self.root_child_pos()]
    }
}
