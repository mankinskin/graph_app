use crate::{
    graph::vertex::wide::Wide,
    path::{
        GetRoleChildPath,
        accessors::role::{
            End,
            Start,
        },
        structs::rooted::{
            index_range::IndexRangePath,
            role_path::{
                IndexEndPath,
                IndexStartPath,
            },
        },
    },
    trace::cache::key::directed::up::UpKey,
};

use super::{
    RoleTraceKey,
    TraceRole,
    cache::{
        key::directed::up::UpPosition,
        new::UpEdit,
    },
    traceable::Traceable,
};

#[derive(Debug)]
pub enum TraceCommand {
    Postfix(PostfixCommand),
    Prefix(PrefixCommand),
    Range(RangeCommand),
}
impl Traceable for TraceCommand {
    fn trace<G: super::has_graph::HasGraph>(
        self,
        ctx: &mut super::TraceContext<G>,
    ) {
        match self {
            Self::Postfix(cmd) => cmd.trace(ctx),
            Self::Prefix(cmd) => cmd.trace(ctx),
            Self::Range(cmd) => cmd.trace(ctx),
        }
    }
}

#[derive(Debug)]
pub struct PostfixCommand {
    pub path: IndexStartPath,
    pub add_edges: bool,
    pub root_up_key: RoleTraceKey<Start>,
}
impl Traceable for PostfixCommand {
    fn trace<G: super::has_graph::HasGraph>(
        self,
        ctx: &mut super::TraceContext<G>,
    ) {
        assert!(!self.path.role_path.sub_path.is_empty());
        let first = self.path.role_path.sub_path.first().unwrap();
        let start_index = *ctx.trav.graph().expect_child_at(first);
        let prev = TraceRole::<Start>::trace_sub_path(
            ctx,
            &self.path,
            UpKey {
                index: start_index.clone(),
                pos: start_index.width().into(),
            },
            self.add_edges,
        );
        let location = self.path.role_root_child_location::<Start>();
        let new = UpEdit {
            target: self.root_up_key.clone(),
            prev,
            location,
        };
        ctx.cache.add_state(new, self.add_edges);
    }
}

#[derive(Debug)]
pub struct PrefixCommand {
    pub path: IndexEndPath,
    pub add_edges: bool,
}
impl Traceable for PrefixCommand {
    fn trace<G: super::has_graph::HasGraph>(
        self,
        ctx: &mut super::TraceContext<G>,
    ) {
        let root_exit = self.path.role_root_child_location::<End>();
        let exit_key = ctx.skip_key(0, 0.into(), root_exit);
        TraceRole::<End>::trace_sub_path(
            ctx,
            &self.path,
            exit_key,
            self.add_edges,
        );
    }
}

#[derive(Debug)]
pub struct RangeCommand {
    pub path: IndexRangePath,
    pub add_edges: bool,
    pub root_pos: UpPosition,
}

impl Traceable for RangeCommand {
    fn trace<G: super::has_graph::HasGraph>(
        self,
        ctx: &mut super::TraceContext<G>,
    ) {
        assert!(!self.path.start.sub_path.is_empty());
        let first = self.path.start.sub_path.first().unwrap();
        let start_index = *ctx.trav.graph().expect_child_at(first);
        let prev = TraceRole::<Start>::trace_sub_path(
            ctx,
            &self.path,
            UpKey {
                index: start_index.clone(),
                pos: start_index.width().into(),
            },
            self.add_edges,
        );
        let root_entry = self.path.role_root_child_location::<Start>();
        let root_entry_index = *ctx.trav.graph().expect_child_at(&root_entry);
        let root_up_key = UpKey {
            index: root_entry_index,
            pos: self.root_pos.clone(),
        };
        let new = UpEdit {
            target: root_up_key.clone(),
            prev,
            location: root_entry,
        };
        ctx.cache.add_state(new, self.add_edges);

        let root_entry = self.path.role_root_child_location::<Start>();
        let root_exit = self.path.role_root_child_location::<End>();
        let exit_key =
            ctx.skip_key(root_entry.sub_index, root_up_key.pos, root_exit);

        TraceRole::<End>::trace_sub_path(
            ctx,
            &self.path,
            exit_key,
            self.add_edges,
        );
    }
}
