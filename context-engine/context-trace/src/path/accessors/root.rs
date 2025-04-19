use crate::{
    graph::vertex::{
        child::Child,
        location::pattern::PatternLocation,
        pattern::Pattern,
    },
    trace::has_graph::HasGraph,
};

pub trait GraphRootPattern: GraphRoot + RootPattern {
    fn root_pattern_location(&self) -> PatternLocation;
    fn graph_root_pattern<'a: 'g, 'g, G: HasGraph + 'a>(
        &self,
        trav: &'g G::Guard<'a>,
    ) -> &'g Pattern {
        trav.expect_pattern_at(self.root_pattern_location())
    }
}

pub trait GraphRoot {
    fn root_parent(&self) -> Child;
}

pub trait PatternRoot {
    fn pattern_root_pattern(&self) -> &Pattern;
}

pub trait RootPattern {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, G: HasGraph + 'a>(
        &'b self,
        trav: &'g G::Guard<'a>,
    ) -> &'g Pattern;
}
#[macro_export]
macro_rules! impl_root {
    {
        RootPattern for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl $crate::path::accessors::root::RootPattern for $target {
            fn root_pattern<
                'a: 'g,
                'b: 'g,
                'g,
                G: $crate::trace::has_graph::HasGraph + 'a
            >(&'b $self_, $trav: &'g G::Guard<'a>) -> &'g $crate::graph::vertex::pattern::Pattern {
                $func
            }
        }
    };
    {
        PatternRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl $crate::path::accessors::root::PatternRoot for $target {
            fn pattern_root_pattern(& $self_) -> &Pattern {
                $func
            }
        }
    };
    {
        GraphRootPattern for $target:ty, $self_:ident => $func:expr
    } => {
        impl GraphRootPattern for $target {
            fn root_pattern_location(& $self_) -> $crate::graph::vertex::location::pattern::PatternLocation {
                $func
            }
        }
    };
    {
        GraphRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl $crate::path::accessors::root::GraphRoot for $target {
            fn root_parent(& $self_) -> $crate::graph::vertex::child::Child {
                $func
            }
        }
    }
}
