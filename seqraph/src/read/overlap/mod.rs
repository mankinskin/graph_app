use match_end::MatchEnd;

use hypercontext_api::{
    graph::vertex::child::Child,
    path::{
        accessors::role::{
            End,
            Start,
        },
        structs::{
            role_path::RolePath,
            rooted::{
                role_path::RootedRolePath,
                root::IndexRoot,
            },
        },
    },
};
pub mod cache;
pub mod chain;
pub mod iterator;
pub mod match_end;
pub mod primer;
