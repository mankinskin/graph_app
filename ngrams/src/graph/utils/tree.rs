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
        TopDown,
        TraversalDirection,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
            VocabEntry,
        },
        NGramId,
        ProcessStatus,
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
        let mut queue: VecDeque<_> =
            TopDown::next_nodes(entry).into_iter().collect();
        let mut tree: ChildTree = Default::default();

        let mut visited: HashSet<_> = Default::default();
        while !queue.is_empty()
        {
            //let mut next_layer: Vec<_> = Default::default();
            while let Some((off, node)) = queue.pop_front()
            {
                if visited.contains(&(off, node))
                {
                    continue;
                }
                visited.insert((off, node));
                // check if covered
                if tree.any_covers(off, node)
                {
                    continue;
                }
                if ctx.labels.contains(&node)
                {
                    tree.insert(off, node);
                }
                else
                {
                    let ne = entry.vocab.get_vertex(&node).unwrap();
                    queue.extend(
                        TopDown::next_nodes(&ne)
                            .into_iter()
                            .map(|(o, c)| (o + off, c)),
                    )
                }
            }
            //queue.extend(next_layer)
        }
        tree
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
