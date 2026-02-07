pub(crate) mod child;
pub(crate) mod parent;
pub(crate) mod frequency;

use child::ChildCoverPass;
use itertools::Itertools;
use pretty_assertions::assert_matches;
use range_ext::intersect::Intersect;
use derivative::Derivative;
use context_trace::{
    graph::{
        vertex::{
            data::{
                VertexData,
                VertexDataBuilder,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToToken,
            },
            key::VertexKey,
            wide::Wide,
            VertexIndex,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
    VertexSet,
};
use std::{
    cmp::{
        Ordering,
        Reverse,
    },
    collections::VecDeque,
    fmt::{
        Display,
        Formatter,
    },
    num::NonZeroUsize,
    ops::Range,
};

use derive_new::new;
use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};

use crate::graph::{
    labelling::LabellingCtx,
    partitions::{
        NodePartitionCtx,
        PartitionsCtx,
    },
    traversal::{
        direction::{
            TopDown,
            TraversalDirection,
        },
        pass::TraversalPass,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
            VocabEntry,
        },
        NGramId,
        ProcessStatus, Vocabulary,
    },
};

#[derive(Debug, Deref, DerefMut, Default, IntoIterator, new)]
pub(crate) struct ChildCover
{
    #[into_iterator(owned, ref)]
    #[deref]
    #[deref_mut]
    entries: HashMap<usize, NGramId>,
}
impl ChildCover
{
    // find largest labelled children
    pub(crate) fn from_key(
        ctx: &LabellingCtx,
        key: VertexKey,
    ) -> Self
    {
        let mut ctx = ChildCoverPass::new(ctx, key);
        ctx.run();
        ctx.cover
    }
    pub(crate) fn as_ranges(&self) -> HashSet<Range<usize>>
    {
        self.entries
            .iter()
            .map(|(off, id)| *off..(off + id.width().0))
            .collect()
    }
    pub(crate) fn any_intersect(&self) -> bool
    {
        let ranges = self.as_ranges();
        ranges
            .iter()
            .cartesian_product(&ranges)
            .any(|(l, r)| l != r && l.does_intersect(r))
    }
    pub(crate) fn any_covers(
        &self,
        off: usize,
        node: impl Wide,
    ) -> bool
    {
        self.iter().any(|(&p, &c)| {
            let node_end = off + node.width();
            let probe_end = p + c.width();
            p <= off && node_end <= probe_end
        })
    }
}
