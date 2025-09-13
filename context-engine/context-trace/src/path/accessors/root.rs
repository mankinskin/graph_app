use crate::{
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::{
                IntoPatternLocation,
                PatternLocation,
            },
        },
        pattern::Pattern,
    },
    path::structs::rooted::root::IndexRoot,
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
impl GraphRootPattern for PatternLocation {
    fn root_pattern_location(&self) -> PatternLocation {
        self.clone()
    }
}
impl GraphRootPattern for ChildLocation {
    fn root_pattern_location(&self) -> PatternLocation {
        self.into_pattern_location()
    }
}
pub trait GraphRoot {
    fn root_parent(&self) -> Child;
}
impl GraphRoot for PatternLocation {
    fn root_parent(&self) -> Child {
        self.parent
    }
}
impl GraphRoot for ChildLocation {
    fn root_parent(&self) -> Child {
        self.parent
    }
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
impl RootPattern for ChildLocation {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, G: HasGraph + 'a>(
        &'b self,
        trav: &'g G::Guard<'a>,
    ) -> &'g Pattern {
        GraphRootPattern::graph_root_pattern::<G>(self, trav)
    }
}
impl RootPattern for PatternLocation {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, G: HasGraph + 'a>(
        &'b self,
        trav: &'g G::Guard<'a>,
    ) -> &'g Pattern {
        GraphRootPattern::graph_root_pattern::<G>(self, trav)
    }
}
impl RootPattern for IndexRoot {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, G: HasGraph + 'a>(
        &'b self,
        trav: &'g G::Guard<'a>,
    ) -> &'g Pattern {
        self.location.root_pattern::<G>(trav)
    }
}
impl RootPattern for Pattern {
    fn root_pattern<'a: 'g, 'b: 'g, 'g, G: HasGraph + 'a>(
        &'b self,
        _trav: &'g G::Guard<'a>,
    ) -> &'g Pattern {
        self
    }
}
#[macro_export]
macro_rules! impl_root {
    {
        $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? RootPattern for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> $crate::path::accessors::root::RootPattern for $target {
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
        $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? PatternRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> $crate::path::accessors::root::PatternRoot for $target {
            fn pattern_root_pattern(& $self_) -> &Pattern {
                $func
            }
        }
    };
    {
        $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? GraphRootPattern for $target:ty, $self_:ident => $func:expr
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> GraphRootPattern for $target {
            fn root_pattern_location(& $self_) -> $crate::graph::vertex::location::pattern::PatternLocation {
                $func
            }
        }
    };
    {
        $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? GraphRoot for $target:ty, $self_:ident => $func:expr
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> $crate::path::accessors::root::GraphRoot for $target {
            fn root_parent(& $self_) -> $crate::graph::vertex::child::Child {
                $func
            }
        }
    }
}
