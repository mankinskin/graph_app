pub mod cache;
pub mod child;
pub mod command;
pub mod has_graph;
pub mod node;
pub mod pattern;
pub mod traceable;

use crate::{
    graph::vertex::location::child::ChildLocation,
    path::{
        GetRoleChildPath,
        accessors::{
            child::root::GraphRootChild,
            has_path::HasRolePath,
            role::{
                PathRole,
                Start,
            },
        },
        mutators::move_path::key::TokenPosition,
    },
    trace::{
        cache::{
            TraceCache,
            key::directed::{
                DirectedKey,
                down::DownKey,
            },
        },
        traceable::Traceable,
    },
};
use cache::{
    key::directed::{
        HasTokenPosition,
        down::DownPosition,
        up::{
            UpKey,
            UpPosition,
        },
    },
    new::{
        DownEdit,
        EditKind,
        UpEdit,
    },
};
use command::TraceCommand;
use has_graph::HasGraph;
use std::fmt::Debug;
use tracing::debug;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum StateDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BottomUp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopDown;
// - trace
//      - End paths: top down
//      - Start paths: bottom up
// - keys are relative to the start index
pub trait TraceRolePath<Role: PathRole>:
    GetRoleChildPath + HasRolePath<Role> + GraphRootChild<Role>
{
}
impl<
    Role: PathRole,
    P: GetRoleChildPath + HasRolePath<Role> + GraphRootChild<Role>,
> TraceRolePath<Role> for P
{
}

// o ->x o ->x o ->x root x-> o x-> o
//
// - Start Up Key: (start, start.width())
// - Up Segment Key: (segment, pos in segment)
// - Root Up Key: (root, entry pos in root)
// - Root Down Key: (root, exit pos in root)
// - Down Segment Key: (segment, exit pos in root)
//
//       Root
//
//     /^        >\
//   Segment    Segment
//    /^           >\
// Start         End

pub trait TraceRole<Role: PathRole> {
    fn trace_sub_path<P: TraceRolePath<Role>>(
        &mut self,
        path: &P,
        prev: RoleTraceKey<Role>,
        add_edges: bool,
    ) -> RoleTraceKey<Role>;
}
impl<'a, G: HasGraph, Role: PathRole> TraceRole<Role> for TraceContext<G> {
    fn trace_sub_path<P: TraceRolePath<Role>>(
        &mut self,
        path: &P,
        prev_key: RoleTraceKey<Role>,
        add_edges: bool,
    ) -> RoleTraceKey<Role> {
        let graph = self.trav.graph();

        path.raw_child_path()
            .into_iter()
            .fold(prev_key, |prev, loc| {
                let key = Role::Direction::build_key(&graph, *prev.pos(), loc);
                debug!("Trace {:#?}", key);
                self.cache.add_state(
                    Role::Direction::build_edit(key.clone(), prev, loc.clone()),
                    add_edges,
                );
                key
            })
    }
}

#[derive(Debug)]
pub struct TraceContext<G: HasGraph> {
    pub trav: G,
    pub cache: TraceCache,
}
impl<G: HasGraph> TraceContext<G> {
    //fn skip_key(
    //    &mut self,
    //    root_entry: usize, // sub index
    //    root_up_pos: UpPosition,
    //    root_exit: ChildLocation,
    //) -> RoleTraceKey<End> {
    //    let graph = self.trav.graph();

    //    let pattern = graph.expect_pattern_at(root_exit.clone());
    //    let root_down_pos = root_up_pos.0
    //        + pattern
    //            .get(root_entry + 1..root_exit.sub_index)
    //            .map(pattern_width)
    //            .unwrap_or_default();

    //    DownKey::new(
    //        *graph.expect_child_at(root_exit.clone()),
    //        root_down_pos.into(),
    //    )
    //}
    pub fn trace_command(
        &mut self,
        command: TraceCommand,
    ) {
        command.trace(self)
    }
}

pub trait TraceKey:
    HasTokenPosition + Debug + Clone + Into<DirectedKey>
{
}
impl<T: HasTokenPosition + Debug + Clone + Into<DirectedKey>> TraceKey for T {}

pub type RoleTraceKey<Role> =
    <<Role as PathRole>::Direction as TraceDirection>::Key;
pub trait TraceDirection {
    type Opposite: TraceDirection;
    type Key: TraceKey;
    type Edit: Into<EditKind>;
    fn build_key<G: HasGraph>(
        trav: &G,
        last_pos: TokenPosition,
        location: &ChildLocation,
    ) -> Self::Key;
    fn build_edit(
        key: Self::Key,
        prev: Self::Key,
        location: ChildLocation,
    ) -> Self::Edit;
}

impl TraceDirection for BottomUp {
    type Opposite = TopDown;
    type Key = UpKey;
    type Edit = UpEdit;
    fn build_key<G: HasGraph>(
        trav: &G,
        last_pos: TokenPosition,
        location: &ChildLocation,
    ) -> Self::Key {
        let delta = location.role_inner_width::<_, Start>(trav);
        UpKey {
            index: location.parent,
            pos: UpPosition::from(last_pos + delta),
        }
    }
    fn build_edit(
        key: Self::Key,
        prev: Self::Key,
        location: ChildLocation,
    ) -> Self::Edit {
        UpEdit {
            target: key.clone(),
            prev,
            location,
        }
    }
}

impl TraceDirection for TopDown {
    type Opposite = BottomUp;
    type Key = DownKey;
    type Edit = DownEdit;
    fn build_key<G: HasGraph>(
        trav: &G,
        last_pos: TokenPosition,
        location: &ChildLocation,
    ) -> Self::Key {
        let graph = trav.graph();
        let index = *graph.expect_child_at(location);
        let delta = graph.expect_child_offset(location);
        DownKey {
            index,
            pos: DownPosition::from(last_pos + delta),
        }
    }
    fn build_edit(
        key: Self::Key,
        prev: Self::Key,
        location: ChildLocation,
    ) -> Self::Edit {
        DownEdit {
            target: key.clone(),
            prev,
            location,
        }
    }
}
