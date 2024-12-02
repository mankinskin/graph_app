use std::{
    borrow::Borrow,
    iter::FromIterator,
    num::NonZeroUsize,
};

use derive_more::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;

use context::*;
use joined::*;
use partition::*;

use crate::{
    graph::HypergraphRef, insert::context::InsertContext, join::{
        context::node::context::NodeJoinContext,
        partition::{
            info::{
                range::role::{
                    In,
                    Join,
                    Post,
                    Pre,
                }, visit::{
                    PartitionBorders,
                    VisitPartition,
                }, JoinPartition
            },
            splits::{
                offset::OffsetSplits, HasPosSplits, PosSplitRef
            },
        },
    }, split::{
        cache::{
            split::Split,
            SplitCache,
        },
        complete::position_splits,
    }, traversal::{
        cache::key::SplitKey,
        folder::state::{
            FoldState,
            RootMode,
        },
        traversable::TraversableMut,
    }, HashMap
};
use crate::graph::vertex::{
    child::Child,
    location::SubLocation,
    wide::Wide,
};

pub mod context;
pub mod delta;
pub mod joined;
pub mod partition;
pub mod splits;

