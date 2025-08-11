use context_trace::{
    self,
    path::{
        accessors::role::{
            End,
            Start,
        },
        structs::role_path::RolePath,
    },
};

#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: RolePath<End>, // location of postfix/overlap in first index
    pub prefix_path: RolePath<Start>, // location of prefix/overlap in second index
}
