use crate::{direction::Right, path::{accessors::{child::{pos::RootChildPosMut, root::GraphRootChild, LeafChild}, has_path::HasRolePath, role::{End, Start}, root::GraphRoot}, BasePath}, traversal::{iterator::policy::NodePath, result::kind::RoleChildPath}};

use super::{append::PathAppend, move_path::root::MoveRootPos};

pub mod from_advanced;
pub mod into_advanced;
pub mod into_primer;

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