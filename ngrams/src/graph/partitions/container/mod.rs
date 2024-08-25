mod builder;

use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use itertools::Itertools;
use pretty_assertions::assert_matches;
use seqraph::graph::vertex::child::Child;
use seqraph::graph::vertex::wide::Wide;
use std::cmp::{
    Ordering,
    Reverse,
};
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use builder::PartitionBuilder;
use crate::graph::partitions::{NodePartitionCtx, PartitionsCtx};

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
        list: impl IntoIterator<Item=(usize, Child)>,
    ) -> Self
    {
        let vec = list.into_iter()
            .sorted_by_key(|&(o, _)| o)
            .collect_vec();
        //println!("{:#?}", vec.iter().map(|&(p, c)| (p, p + c.width())).collect_vec());

        assert!(!vec.is_empty(), "Can not build a container from empty list!");
        vec.iter().tuple_windows().for_each(|((prev,_), (pos, _))|
            assert!(prev < pos, "{} < {}", prev, pos,)
        );
        let mut iter = vec.into_iter();
        let first = iter.next().unwrap();
        assert_eq!(first.0, 0);
        let mut builder = PartitionBuilder::new(ctx, first.0, first.1);
        while let Some((pos, index)) = iter.next()
        {
            builder.append_child(pos, index);
        }
        Self {
            wall: builder.close(),
        }
    }
}
impl Display for PartitionContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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