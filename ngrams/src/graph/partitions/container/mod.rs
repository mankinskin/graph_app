mod builder;

use builder::PartitionBuilder;
use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
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

#[derive(Debug, Copy, Clone)]
pub enum PartitionCell
{
    ChildIndex(Child),
    GapSize(NonZeroUsize),
}
impl PartitionCell
{
    pub fn width(&self) -> usize
    {
        match self
        {
            Self::ChildIndex(c) => c.width(),
            Self::GapSize(o) => o.get(),
        }
    }
}
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

#[derive(Debug, IntoIterator, Deref)]
pub struct PartitionContainer
{
    wall: Vec<Vec<Child>>,
}
impl PartitionContainer
{
    pub fn from_entry(
        ctx: &PartitionsCtx<'_>,
        entry: &VertexCtx,
    ) -> Self
    {
        // find all largest children
        let tree = ChildTree::from_entry(ctx, entry);
        assert!(
            match entry.width()
            {
                1 => tree.is_empty(),
                _ => !tree.is_empty(),
            }
        );

        // build container with gaps
        //let next = tree.iter().map(|(_, c)| c.vertex_index()).collect();
        let ctx = NodePartitionCtx::new(
            NGramId::new(entry.data.vertex_key(), entry.data.width()),
            ctx,
        );
        Self::from_child_list(&ctx, tree)
    }
    pub fn from_child_list(
        ctx: &NodePartitionCtx,
        list: impl IntoIterator<Item = (usize, NGramId)>,
    ) -> Self
    {
        let child_vec =
            list.into_iter().sorted_by_key(|&(o, _)| o).collect_vec();
        //println!("{:#?}", vec.iter().map(|&(p, c)| (p, p + c.width())).collect_vec());

        assert!(
            !child_vec.is_empty(),
            "Can not build a container from empty list!"
        );
        child_vec
            .iter()
            .tuple_windows()
            .for_each(|((prev, _), (pos, _))| {
                assert!(prev < pos, "{} < {}", prev, pos,)
            });
        let mut child_iter = child_vec.into_iter();
        let first = child_iter.next().unwrap();
        assert_eq!(first.0, 0);
        let mut builder = PartitionBuilder::new(ctx, first.0, first.1);
        for (pos, key) in child_iter
        {
            let index = ctx
                .vocab
                .containment
                .expect_index_for_key(&key.vertex_key());
            builder.append_child(pos, Child::new(index, key.width()));
        }
        Self {
            wall: builder.close(),
        }
    }
}
impl Display for PartitionContainer
{
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result
    {
        for line in &self.wall
        {
            for cell in line
            {
                let (t, s) = ("ch", cell.width());
                write!(f, "{}({})", t, s);
            }
            writeln!(f);
        }
        Ok(())
    }
}
