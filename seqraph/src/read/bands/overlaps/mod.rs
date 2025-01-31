use std::{
    borrow::Borrow,
    num::NonZeroUsize,
    ops::ControlFlow,
};

use tap::Tap;
use tracing::instrument;

use hypercontext_api::{
    direction::r#match::MatchDirection,
    graph::{
        getters::vertex::VertexSet,
        kind::DefaultDirection,
        vertex::{
            child::Child,
            location::child::ChildLocation,
            pattern::Pattern,
            wide::Wide,
        },
    },
    path::{
        accessors::{has_path::HasRolePath, role::{End, Start}},
        mutators::append::PathAppend,
        structs::{
            overlap_primer::OverlapPrimer,
            query_range_path::PatternPrefixPath,
            role_path::RolePath,
            rooted_path::{RootedRolePath, SubPath},
        },
    },
    traversal::{
        iterator::bands::{
            BandIterator,
            PostfixIterator,
        },
        traversable::{
            Traversable,
            TraversableMut,
        },
    },
};
use crate::{
    insert::{
        HasInsertContext, IndexSplitResult
    },
    read::{
        bands::{
            band::{
                BandEnd,
                OverlapBand,
                OverlapBundle,
            },
            overlaps::overlap::{
                cache::OverlapCache,
                Overlap,
                OverlapLink,
            },
        },
        reader::context::ReadContext,
    },
};

pub mod overlap;
