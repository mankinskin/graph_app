use crate::{
    path::{
        accessors::{
            child::RootChildPos,
            role::{
                End,
                PathRole,
                Start,
            },
            root::{
                GraphRootPattern,
                PatternRoot,
            },
        },
        structs::{
            match_end::{
                MatchEnd,
                MatchEndPath,
            },
            query_range_path::QueryRangePath,
            rooted_path::{
                IndexRoot,
                RootedPath,
                RootedRolePath,
                RootedSplitPath,
                RootedSplitPathRef,
                SearchPath,
            },
        },
    }, traversal::{
        state::query::QueryState, traversable::Traversable
    }
};
use auto_impl::auto_impl;
use crate::graph::vertex::{
    child::Child,
    location::child::ChildLocation,
    pattern::{
        pattern_post_ctx_width,
        pattern_pre_ctx_width,
    },
};

#[auto_impl(&, & mut)]
pub trait RootChild<R>: RootChildPos<R> {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child;
}
macro_rules! impl_child {
    {
        RootChild for $target:ty, $self_:ident, $trav:ident => $func:expr
    } => {
        impl<R: PathRole> RootChild<R> for $target
            where $target: RootChildPos<R>
        {
            fn root_child<
                Trav: Traversable,
            >(& $self_, $trav: &Trav) -> Child {
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
impl_child! { RootChild for QueryRangePath, self, _trav => self.root[self.root_child_pos()] }
//impl_child! { RootChild for PatternPrefixPath, self, trav => self.root_child(trav) }
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
impl RootChild<Start> for SearchPath {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        trav.graph().expect_child_at(
            self.path_root()
                .location
                .to_child_location(self.start.sub_path.root_entry),
        )
    }
}

impl RootChild<End> for SearchPath {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        trav.graph().expect_child_at(
            self.path_root()
                .location
                .to_child_location(self.end.sub_path.root_entry),
        )
    }
}
//impl_child! { RootChild for PathLeaf, self, trav => self.root_child(trav) }
impl_child! { RootChild for RootedRolePath<R>, self, trav =>
    trav.graph().expect_child_at(self.path_root().location.to_child_location(RootChildPos::<R>::root_child_pos(&self.role_path)))
}

impl<R: PathRole> RootChild<R> for QueryState
where
    Self: RootChildPos<R>,
{
    fn root_child<Trav: Traversable>(
        &self,
        _trav: &Trav,
    ) -> Child {
        self.pattern_root_child()
    }
}

impl<P: MatchEndPath> RootChild<Start> for MatchEnd<P> {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Path(path) => path.root_child(trav),
        }
    }
}

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
impl GraphRootChild<Start> for SearchPath {
    fn root_child_location(&self) -> ChildLocation {
        self.root.location.to_child_location(self.start.root_entry)
    }
}

impl GraphRootChild<End> for SearchPath {
    fn root_child_location(&self) -> ChildLocation {
        self.root.location.to_child_location(self.end.root_entry)
    }
}

impl<R: PathRole> GraphRootChild<R> for RootedRolePath<R, IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.role_path.sub_path.root_entry)
    }
}

impl<R: PathRole> GraphRootChild<R> for RootedSplitPath<IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.sub_path.root_entry)
    }
}

impl<R: PathRole> GraphRootChild<R> for RootedSplitPathRef<'_, IndexRoot> {
    fn root_child_location(&self) -> ChildLocation {
        self.path_root()
            .location
            .to_child_location(self.sub_path.root_entry)
    }
}
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

impl<R> PatternRootChild<R> for QueryRangePath where Self: RootChildPos<R> {}

impl<R> PatternRootChild<R> for QueryState where Self: RootChildPos<R> {}
//impl<R> PatternRootChild<R> for PatternPrefixPath
//    where PatternPrefixPath: RootChildPos<R>
//{
//}
//impl PatternRootChild<End> for OverlapPrimer {
//}
