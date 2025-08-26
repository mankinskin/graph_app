use context_trace::path::structs::rooted::role_path::{
    IndexEndPath,
    IndexStartPath,
};

#[derive(Debug)]
pub struct ExpansionLink {
    pub expansion_prefix: IndexStartPath,
    pub root_postfix: IndexEndPath,
    pub start_bound: usize,
}
