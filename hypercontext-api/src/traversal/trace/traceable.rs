use crate::{
    path::{
        accessors::role::Start,
        mutators::move_path::key::TokenPosition,
        RoleChildPath,
    },
    traversal::{
        state::top_down::end::{
            EndKind,
            EndState,
            PrefixEnd,
            RangeEnd,
        },
        traversable::Traversable,
    },
};

use super::context::TraceContext;

pub trait Traceable {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<Trav>,
    );
}
impl Traceable for EndState {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<Trav>,
    ) {
        match &self.kind {
            EndKind::Range(p) => (self.root_pos, p).trace(ctx),
            EndKind::Prefix(p) => (self.root_pos, p).trace(ctx),
            _ => {}
        }
    }
}
impl Traceable for (TokenPosition, &RangeEnd) {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<Trav>,
    ) {
        let &(root_pos, end) = self;
        let root_entry = end.path.role_root_child_location::<Start>().sub_index;
        ctx.trace_path(root_entry, &end.path, root_pos, true)
    }
}
impl Traceable for (TokenPosition, &PrefixEnd) {
    fn trace<Trav: Traversable>(
        &self,
        ctx: &mut TraceContext<Trav>,
    ) {
        let &(root_pos, end) = self;
        ctx.trace_path(0, &end.path, root_pos, true)
    }
}
