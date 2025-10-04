use context_trace::*;

#[derive(Debug)]
pub struct ExpansionLink {
    pub expansion_prefix: IndexStartPath,
    pub root_postfix: IndexEndPath,
    pub start_bound: usize,
}
