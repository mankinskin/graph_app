use super::container::{
    extend::ExtendStates,
    pruning::{
        PruneStates,
        PruningMap,
        PruningState,
    },
    StateContainer,
};
use crate::{
    graph::{
        getters::NoMatch,
        vertex::{
            child::Child,
            pattern::IntoPattern,
            wide::Wide,
        },
    },
    traversal::{
        cache::key::root::RootKey,
        state::traversal::TraversalState,
        traversable::Traversable,
    },
};
use itertools::Itertools;
use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::ControlFlow,
};

use super::{
    cache::{
        key::UpKey,
        TraversalCache,
    },
    fold::FoldFinished,
    iterator::policy::DirectedTraversalPolicy,
    result::TraversalResult,
    state::{
        end::EndState,
        query::QueryState,
        start::StartState,
        ApplyStatesCtx,
    },
    traversable::TravKind,
};

//pub mod state;

