use super::{
    append::PathAppend,
    move_path::root::MoveRootPos,
};
use crate::{
    direction::Right,
    path::{
        BasePath,
        RoleChildPath,
        accessors::{
            child::{
                LeafChild,
                RootChildPosMut,
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
    trace::traversable::Traversable,
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
    RoleChildPath
    + NodePath<Start>
    + BasePath
    + HasRolePath<Start>
    + HasRolePath<End>
    + GraphRootChild<Start>
    + GraphRootChild<End>
    + LeafChild<Start>
    + LeafChild<End>
    + MoveRootPos<Right, End>
    + RootChildPosMut<End>
    + GraphRoot
    + PathAppend
{
}

impl<
    T: RoleChildPath
        + NodePath<Start>
        + BasePath
        + HasRolePath<Start>
        + HasRolePath<End>
        + GraphRootChild<Start>
        + GraphRootChild<End>
        + LeafChild<Start>
        + LeafChild<End>
        + MoveRootPos<Right, End>
        + RootChildPosMut<End>
        + PathAppend,
> Advanced for T
{
}
pub trait FromAdvanced<A: Advanced> {
    fn from_advanced<Trav: Traversable>(
        path: A,
        trav: &Trav,
    ) -> Self;
}
pub trait IntoAdvanced: Sized + Clone {
    type Next;
    fn into_advanced<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Result<Self::Next, Self>;
}
