use context_trace::{
    path::{
        mutators::move_path::key::TokenPosition,
        structs::rooted::role_path::IndexStartPath,
    },
    trace::{
        cache::key::directed::up::UpKey,
        command::PostfixCommand,
        has_graph::HasGraph,
        traceable::Traceable,
        TraceContext,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostfixEnd {
    pub path: IndexStartPath,
    //pub inner_width: usize,
    pub root_pos: TokenPosition,
}
impl Traceable for PostfixEnd {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceContext<G>,
    ) {
        PostfixCommand::from(self).trace(ctx)
    }
}

impl From<PostfixEnd> for PostfixCommand {
    fn from(value: PostfixEnd) -> Self {
        PostfixCommand {
            add_edges: true,
            path: value.path.clone(),
            root_up_key: UpKey::new(
                value.path.root.location.parent.into(),
                value.root_pos.into(),
            ),
        }
    }
}

impl PostfixEnd {}
