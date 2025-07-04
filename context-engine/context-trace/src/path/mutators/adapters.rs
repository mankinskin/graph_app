use super::{
    append::PathAppend,
    move_path::root::MoveRootIndex,
};
use crate::{
    direction::Right,
    path::{
        BasePath,
        RolePathUtils,
        accessors::{
            child::{
                LeafChild,
                RootChildIndexMut,
                root::{
                    GraphRootChild,
                    RootChild,
                },
            },
            has_path::HasRolePath,
            role::{
                End,
                PathRole,
                Start,
            },
            root::GraphRoot,
        },
    },
    trace::has_graph::HasGraph,
};
use std::fmt::Debug;

pub trait NodePath<R: PathRole>:
    RootChild<R> + Send + Clone + Eq + Debug
{
}

impl<R: PathRole, T: RootChild<R> + Send + Clone + Eq + Debug> NodePath<R>
    for T
{
}

pub trait Advanced:
    RolePathUtils
    + NodePath<Start>
    + BasePath
    + HasRolePath<Start>
    + HasRolePath<End>
    + GraphRootChild<Start>
    + GraphRootChild<End>
    + LeafChild<Start>
    + LeafChild<End>
    + MoveRootIndex<Right, End>
    + RootChildIndexMut<End>
    + GraphRoot
    + PathAppend
{
}

impl<
    T: RolePathUtils
        + NodePath<Start>
        + BasePath
        + HasRolePath<Start>
        + HasRolePath<End>
        + GraphRootChild<Start>
        + GraphRootChild<End>
        + LeafChild<Start>
        + LeafChild<End>
        + MoveRootIndex<Right, End>
        + RootChildIndexMut<End>
        + PathAppend,
> Advanced for T
{
}
pub trait FromAdvanced<A: Advanced> {
    fn from_advanced<G: HasGraph>(
        path: A,
        trav: &G,
    ) -> Self;
}
pub trait IntoAdvanced: Sized + Clone {
    type Next;
    fn into_advanced<G: HasGraph>(
        self,
        trav: &G,
    ) -> Result<Self::Next, Self>;
}
