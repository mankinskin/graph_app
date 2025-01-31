use crate::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
        pattern::{
            pattern_post_ctx_width,
            pattern_pre_ctx_width,
        },
    },
    path::{
        accessors::{
            child::RootChildPos,
            role::PathRole,
            root::{
                GraphRootPattern,
                PatternRoot,
            },
        },
        structs::query_range_path::FoldablePath,
    },
    traversal::{
        state::cursor::PathCursor,
        traversable::Traversable,
    },
};
use auto_impl::auto_impl;

#[auto_impl(&, & mut)]
pub trait RootChild<R>: RootChildPos<R> {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
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
                Trav: Traversable,
            >(& $self_, $trav: &Trav) -> $crate::graph::vertex::child::Child {
                $func
            }
        }
    };
}
//impl<R: PathRole, P: RootChild<R>> RootChild<R> for &'_ P
//    where P: RootChildPos<R>
//{
//    fn root_child<
//        Trav: Traversable,
//    >(&self, trav: &Trav) -> Child {
//        (*self).root_child(trav)
//    }
//}
//impl<R: PathRole, P: RootChild<R>> RootChild<R> for &'_ mut P
//    where P: RootChildPos<R>
//{
//    fn root_child<
//        Trav: Traversable,
//    >(&self, trav: &Trav) -> Child {
//        (*self).root_child(trav)
//    }
//}
//impl RootChild<End> for OverlapPrimer {
//    fn root_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>
//    >(&self, trav: &Trav) -> Child {
//        self.start
//    }
//}
impl<R: PathRole, P: RootChild<R> + FoldablePath> RootChild<R> for PathCursor<P> {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        self.path.root_child(trav)
    }
}
//impl_child! { RootChild for PathLeaf, self, trav => self.root_child(trav) }

//impl<R: PathRole> RootChild<R> for RangeCursor
//where
//    Self: RootChildPos<R>,
//{
//    fn root_child<Trav: Traversable>(
//        &self,
//        _trav: &Trav,
//    ) -> Child {
//        self.pattern_root_child()
//    }
//}

/// used to get a direct child in a Graph
pub trait GraphRootChild<R>: GraphRootPattern + RootChildPos<R> {
    fn root_child_location(&self) -> ChildLocation;
    fn graph_root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        trav.graph()
            .expect_child_at(<_ as GraphRootChild<R>>::root_child_location(self))
    }
    fn root_post_ctx_width<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> usize {
        let i = self.root_child_location().sub_index;
        let g = trav.graph();
        let p = self.graph_root_pattern::<Trav>(&g);
        pattern_post_ctx_width(p, i)
    }
    fn root_pre_ctx_width<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> usize {
        let i = self.root_child_location().sub_index;
        let g = trav.graph();
        let p = self.graph_root_pattern::<Trav>(&g);
        pattern_pre_ctx_width(p, i)
    }
}

//impl<R, P: GraphRootChild<R>> GraphRootChild<R> for OriginPath<P> {
//    fn root_child_location(&self) -> ChildLocation {
//        <_ as GraphRootChild::<R>>::root_child_location(&self.postfix)
//    }
//}
//impl<R> GraphRootChild<R> for PathLeaf {
//    fn root_child_location(&self) -> ChildLocation {
//        self.entry
//    }
//}

/// used to get a direct child of a pattern
pub trait PatternRootChild<R>: RootChildPos<R> + PatternRoot {
    fn pattern_root_child(&self) -> Child {
        PatternRoot::pattern_root_pattern(self)[self.root_child_pos()]
    }
}

//impl<R> PatternRootChild<R> for RangeCursor where Self: RootChildPos<R> {}
//impl PatternRootChild<End> for OverlapPrimer {
//}
