pub mod pos;

use auto_impl::auto_impl;
use pos::*;

pub mod root;

use crate::{
    traversal::{
        context::QueryStateContext,
        path::{
            accessors::{
                has_path::{
                    HasPath,
                    HasRolePath,
                },
                role::{
                    End,
                    PathRole,
                },
            },
            structs::{
                query_range_path::QueryRangePath,
                role_path::RolePath,
                rooted_path::SearchPath,
            },
        },
        traversable::Traversable,
    },
    vertex::{
        child::Child,
        location::child::ChildLocation,
    },
};
use root::*;

pub trait LeafChild<R>: RootChildPos<R> {
    fn leaf_child_location(&self) -> Option<ChildLocation>;
    fn leaf_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child;
}

impl<R: PathRole, P: RootChild<R> + PathChild<R>> LeafChild<R> for P {
    fn leaf_child_location(&self) -> Option<ChildLocation> {
        self.path_child_location()
    }
    fn leaf_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        self.path_child(trav)
            .unwrap_or_else(|| self.root_child(trav))
    }
}

pub trait LeafChildPosMut<R>: RootChildPosMut<R> {
    fn leaf_child_pos_mut(&mut self) -> &mut usize;
}

//impl<R: PathRole, P: PathChild<R> + RootChildPosMut<R>> LeafChildPosMut<R> for P {
//    fn leaf_child_pos_mut(&mut self) -> &mut usize {
//        if let Some(loc) = self.path_child_location_mut() {
//            &mut loc.sub_index
//        } else {
//            self.root_child_pos_mut()
//        }
//    }
//}
impl LeafChildPosMut<End> for QueryStateContext<'_> {
    fn leaf_child_pos_mut(&mut self) -> &mut usize {
        self.state.end.leaf_child_pos_mut()
    }
}

impl LeafChildPosMut<End> for RolePath<End> {
    fn leaf_child_pos_mut(&mut self) -> &mut usize {
        if !self.path().is_empty() {
            &mut self.path_child_location_mut().unwrap().sub_index
        } else {
            self.root_child_pos_mut()
        }
    }
}

/// used to get a descendant in a Graph, pointed to by a child path
#[auto_impl(& mut)]
pub trait PathChild<R: PathRole>: HasPath<R> {
    fn path_child_location(&self) -> Option<ChildLocation> {
        R::bottom_up_iter(self.path().iter()).next().cloned() as Option<_>
    }
    fn path_child_location_mut(&mut self) -> Option<&mut ChildLocation> {
        R::bottom_up_iter(self.path_mut().iter_mut()).next()
    }
    fn path_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Option<Child> {
        self.path_child_location()
            .map(|loc| trav.graph().expect_child_at(loc))
    }
}

//impl<R: PathRole, P: PathChild<R>> PathChild<R> for &'_ P
//    where Self: HasPath<R> + PatternRootChild<R>
//{
//}
//impl<R: PathRole, P: PathChild<R>> PathChild<R> for &'_ mut P
//    where Self: HasPath<R> + PatternRootChild<R>
//{
//}
impl<R: PathRole> PathChild<R> for QueryRangePath where Self: HasPath<R> + PatternRootChild<R> {}

impl<R: PathRole> PathChild<R> for QueryStateContext<'_> where Self: HasPath<R> + PatternRootChild<R>
{}

impl<R: PathRole> PathChild<R> for RolePath<R> {}

impl<R: PathRole> PathChild<R> for SearchPath
    where
        SearchPath: HasRolePath<R>,
{
    fn path_child_location(&self) -> Option<ChildLocation> {
        Some(
            R::bottom_up_iter(self.path().iter())
                .next()
                .cloned()
                .unwrap_or(
                    self.root
                        .location
                        .to_child_location(self.role_path().root_entry),
                ),
        )
    }
    fn path_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Option<Child> {
        PathChild::<R>::path_child(self.role_path(), trav)
    }
}
//impl PathChild<End> for OverlapPrimer {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        self.context.path_child(trav)
//    }
//}
//impl PathChild<End> for PatternPrefixPath {
//    fn path_child<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &'a Trav) -> Option<Child> {
//        if let Some(end) = &self.end {
//            PathChild::<End>::path_child(end, trav)
//        } else {
//            Some(RootChild::<End>::root_child(self, trav))
//        }
//    }
//}
//impl<R: PathRole, P: PathChild<R>> PathChild<R> for OriginPath<P> {
//    fn path_child<
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(&self, trav: &Trav) -> Option<Child> {
//        self.postfix.path_child(trav)
//    }
//}
