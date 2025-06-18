use context_trace::{
    graph::vertex::location::child::ChildLocation,
    path::{
        mutators::move_path::key::TokenPosition,
        structs::rooted::index_range::IndexRangePath,
    },
    trace::{
        cache::key::{
            directed::down::DownKey,
            props::LeafKey,
        },
        command::RangeCommand,
        has_graph::HasGraph,
        traceable::Traceable,
        TraceCtx,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeEnd {
    pub path: IndexRangePath,
    pub target: DownKey,
    pub root_pos: TokenPosition,
}
impl LeafKey for RangeEnd {
    fn leaf_location(&self) -> ChildLocation {
        self.path.leaf_location()
    }
}

impl Traceable for &RangeEnd {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    ) {
        RangeCommand::from(self).trace(ctx)
    }
}

impl From<&RangeEnd> for RangeCommand {
    fn from(value: &RangeEnd) -> Self {
        RangeCommand {
            add_edges: true,
            path: value.path.clone(),
            root_pos: value.root_pos.into(),
        }
    }
}
