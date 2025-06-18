use super::{
    BottomUp,
    RoleTraceKey,
    TopDown,
    TraceCtx,
    TraceRole,
    cache::{
        key::directed::{
            down::DownKey,
            up::UpPosition,
        },
        new::NewTraceEdge,
    },
    has_graph::HasGraph,
    traceable::Traceable,
};
use crate::{
    graph::vertex::wide::Wide,
    path::{
        RolePathUtils,
        accessors::{
            role::{
                End,
                Start,
            },
            root::RootPattern,
        },
        structs::{
            role_path::CalcOffset,
            rooted::{
                index_range::IndexRangePath,
                role_path::{
                    IndexEndPath,
                    IndexStartPath,
                },
            },
        },
    },
    trace::cache::key::directed::up::UpKey,
};
use tracing::debug;
#[derive(Debug)]
pub enum TraceCommand {
    Postfix(PostfixCommand),
    Prefix(PrefixCommand),
    Range(RangeCommand),
}
impl Traceable for TraceCommand {
    fn trace<G: super::has_graph::HasGraph>(
        self,
        ctx: &mut super::TraceCtx<G>,
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
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    ) {
        let first = self.path.role_leaf_child_location::<Start>().unwrap();
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
        debug!("{:#?}", self.root_up_key);
        let new = NewTraceEdge::<BottomUp> {
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
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    ) {
        let root_exit = self.path.role_root_child_location::<End>();
        // TODO: implement root_child for prefix/postfix path with most outer root child
        let exit_key = DownKey {
            pos: (self.path.root_pattern::<G>(&ctx.trav.graph())[0].width()
                + self.path.calc_offset(&ctx.trav))
            .into(),
            index: root_exit.parent,
            //*ctx.trav.graph().expect_child_at(root_exit.clone()),
        };
        let target = DownKey {
            index: *ctx.trav.graph().expect_child_at(root_exit.clone()),
            pos: exit_key.pos,
        };
        debug!("{:#?}", target);
        let new = NewTraceEdge::<TopDown> {
            target: target.clone(),
            prev: exit_key.clone(),
            location: root_exit,
        };
        ctx.cache.add_state(new, self.add_edges);

        TraceRole::<End>::trace_sub_path(
            ctx,
            &self.path,
            target,
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
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    ) {
        let first = self.path.role_leaf_child_location::<Start>().unwrap();
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
        //let location = self.path.role_root_child_location::<Start>();
        //let new = UpEdit {
        //    target: self.root_up_key.clone(),
        //    prev,
        //    location,
        //};
        let root_entry = self.path.role_root_child_location::<Start>();
        //let root_entry_index = *ctx.trav.graph().expect_child_at(&root_entry);
        let root_up_key = UpKey {
            index: root_entry.parent,
            pos: self.root_pos.clone(),
        };
        debug!("{:#?}", root_up_key);
        let new = NewTraceEdge::<BottomUp> {
            target: root_up_key.clone(),
            prev,
            location: root_entry,
        };
        ctx.cache.add_state(new, self.add_edges);

        let root_exit = self.path.role_root_child_location::<End>();

        let exit_key = DownKey {
            pos: (self.path.role_leaf_child::<Start, _>(&ctx.trav).width()
                + self.path.calc_offset(&ctx.trav))
            .into(),
            index: root_exit.parent,
        };
        let target_key = DownKey {
            pos: exit_key.pos,
            index: *ctx.trav.graph().expect_child_at(root_exit.clone()),
        };
        debug!("{:#?}", target_key);
        let new = NewTraceEdge::<TopDown> {
            target: target_key.clone(),
            prev: exit_key.clone(),
            location: root_exit,
        };
        ctx.cache.add_state(new, self.add_edges);

        TraceRole::<End>::trace_sub_path(
            ctx,
            &self.path,
            target_key,
            self.add_edges,
        );
    }
}
