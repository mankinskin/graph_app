mod builder;

use crate::graph::{
    partitions::{
        NodePartitionCtx,
        PartitionsCtx,
    },
    vocabulary::NGramId,
};
use builder::PartitionBuilder;
use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use itertools::Itertools;
use pretty_assertions::assert_matches;
use seqraph::graph::vertex::{
    child::Child,
    has_vertex_key::HasVertexKey,
    wide::Wide,
};
use std::{
    cmp::{
        Ordering,
        Reverse,
    },
    fmt::{
        Display,
        Formatter,
    },
    num::NonZeroUsize,
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

#[derive(Debug, IntoIterator, Deref)]
pub struct PartitionContainer
{
    wall: Vec<Vec<Child>>,
}
impl PartitionContainer
{
    pub fn from_child_list(
        ctx: &NodePartitionCtx,
        list: impl IntoIterator<Item = (usize, NGramId)>,
    ) -> Self
    {
        let child_vec = list.into_iter().sorted_by_key(|&(o, _)| o).collect_vec();
        //println!("{:#?}", vec.iter().map(|&(p, c)| (p, p + c.width())).collect_vec());

        assert!(
            !child_vec.is_empty(),
            "Can not build a container from empty list!"
        );
        child_vec.iter()
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
            let index = ctx.vocab.containment.expect_index_for_key(&key.vertex_key());
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
                //let (t, s) = match cell
                //{
                //    PartitionCell::GapSize(s) => ("gp", s.get()),
                //    PartitionCell::ChildIndex(c) => ("ch", c.width()),
                //};
                let (t, s) = ("ch", cell.width());
                write!(f, "{}({})", t, s);
            }
            writeln!(f);
            //println!("{:#?}", line)
            //self.labels.insert(c);
        }
        Ok(())
    }
}
