pub mod child_cover;
pub mod child_dedup;

use child_cover::ChildCoverPass;
use itertools::Itertools;
use pretty_assertions::assert_matches;
use range_ext::intersect::Intersect;
use seqraph::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            data::{
                VertexData,
                VertexDataBuilder,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            has_vertex_key::HasVertexKey,
            wide::Wide,
            VertexIndex,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
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

#[derive(Debug, Deref, DerefMut, Default, IntoIterator)]
pub struct ChildTree
{
    #[deref]
    #[deref_mut]
    #[into_iterator(owned, ref)]
    entries: HashMap<usize, NGramId>,
}
impl ChildTree
{
    // find largest labelled children
    pub fn from_entry(
        ctx: &LabellingCtx,
        entry: &VertexCtx<'_>,
    ) -> Self
    {
        let mut ctx = ChildCoverPass::new(ctx, entry);
        ctx.run();
        ctx.tree
    }
    pub fn as_ranges(&self) -> HashSet<Range<usize>>
    {
        self.entries
            .iter()
            .map(|(off, id)| *off..(off + id.width()))
            .collect()
    }
    pub fn any_intersect(&self) -> bool
    {
        let ranges = self.as_ranges();
        ranges
            .iter()
            .cartesian_product(&ranges)
            .any(|(l, r)| l != r && l.does_intersect(r))
    }
    pub fn any_covers(
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
