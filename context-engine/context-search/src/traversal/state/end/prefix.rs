use context_trace::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixEnd {
    pub path: IndexEndPath,
    pub target: DownKey,
}
impl From<&PrefixEnd> for PrefixCommand {
    fn from(value: &PrefixEnd) -> Self {
        PrefixCommand {
            add_edges: true,
            path: value.path.clone(),
        }
    }
}
impl Traceable for &PrefixEnd {
    fn trace<G: HasGraph>(
        self,
        ctx: &mut TraceCtx<G>,
    ) {
        PrefixCommand::from(self).trace(ctx)
    }
}
