use crate::{
    direction::Right,
    graph::vertex::location::child::ChildLocation,
    path::{
        accessors::{
            child::{
                root::GraphRootChild,
                LeafChild,
                RootChildPosMut,
            },
            has_path::HasRolePath,
            role::{
                End,
                Start,
            },
            root::GraphRoot,
        },
        BasePath,
        RoleChildPath,
    },
    traversal::{
        iterator::policy::NodePath,
        state::parent::ParentState,
        traversable::Traversable,
    },
};

use super::{
    append::PathAppend,
    move_path::root::MoveRootPos,
};

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

pub trait IntoPrimer: Sized {
    fn into_primer<Trav: Traversable>(
        self,
        trav: &Trav,
        parent_entry: ChildLocation,
    ) -> ParentState;
}
