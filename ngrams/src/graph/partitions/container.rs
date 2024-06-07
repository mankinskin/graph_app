use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use itertools::Itertools;
use pretty_assertions::assert_matches;
use seqraph::vertex::child::Child;
use seqraph::vertex::wide::Wide;
use std::cmp::{
    Ordering,
    Reverse,
};
use std::num::NonZeroUsize;

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
#[derive(Default, Debug, Deref, DerefMut)]
struct PartitionLineBuilder
{
    pos: usize,
    #[deref]
    #[deref_mut]
    line: Vec<PartitionCell>,
}
impl PartitionLineBuilder
{
    pub fn append(
        &mut self,
        index: Child,
    )
    {
        self.line.push(PartitionCell::ChildIndex(index));
        self.pos += index.width();
    }
    pub fn skip(
        &mut self,
        offset: NonZeroUsize,
    )
    {
        self.line.push(PartitionCell::GapSize(offset));
        self.pos += offset.get();
    }
    pub fn append_after_offset(
        &mut self,
        offset: Option<NonZeroUsize>,
        index: Child,
    )
    {
        if let Some(non_zero) = offset
        {
            self.skip(non_zero);
        }
        self.append(index);
    }
    pub fn offset_to(
        &self,
        pos: usize,
    ) -> Option<NonZeroUsize>
    {
        NonZeroUsize::new(pos - self.pos)
    }
    pub fn skip_to(
        &mut self,
        end_pos: usize,
    )
    {
        if let Some(o) = self.offset_to(end_pos)
        {
            self.skip(o);
        }
    }
    pub fn end_pos(&self) -> usize
    {
        let cell = self.line.last().unwrap();
        let cell_width = cell.width();
        self.pos + cell_width
    }
    pub fn close(
        mut self,
        end_pos: usize,
    ) -> Vec<PartitionCell>
    {
        self.skip_to(end_pos);
        self.line
    }
}

#[derive(Debug, Clone, Copy, Hash)]
struct PartitionCursor
{
    pub line: usize,
}
impl PartitionCursor
{
    pub fn get_current_line<'a>(
        &self,
        builder: &'a mut PartitionBuilder,
    ) -> &'a mut PartitionLineBuilder
    {
        builder.get_line_mut(self.line)
    }
}

#[derive(Debug)]
struct PartitionBuilder
{
    cursor: PartitionCursor,
    wall: Vec<PartitionLineBuilder>,
}

impl PartitionBuilder
{
    pub fn get_line_mut(
        &mut self,
        index: usize,
    ) -> &mut PartitionLineBuilder
    {
        self.wall.get_mut(index).expect("Invalid PartitionCursor")
    }
    pub fn get_current_line(&mut self) -> &mut PartitionLineBuilder
    {
        self.get_line_mut(self.cursor.line)
    }

    pub fn new(
        offset: usize,
        index: Child,
    ) -> Self
    {
        let mut builder = Self {
            cursor: PartitionCursor { line: 0 },
            wall: Default::default(),
        };
        builder.append_child(offset, index);
        builder
    }
    pub fn create_line(
        &mut self,
        pos: usize,
        index: Child,
    )
    {
        let mut line = PartitionLineBuilder::default();
        line.append_after_offset(NonZeroUsize::new(pos), index);
        self.cursor.line = self.wall.len();
        self.wall.push(line);
    }
    pub fn create_and_append_line(
        &mut self,
        pos: usize,
        index: Child,
    )
    {
        let mut line = PartitionLineBuilder::default();
        line.append_after_offset(NonZeroUsize::new(pos), index);
        self.cursor.line = self.wall.len();
        self.wall.push(line);
    }
    pub fn append_at_line(
        &mut self,
        line_index: usize,
        pos: usize,
        index: Child,
    )
    {
        let line = self.get_line_mut(line_index);
        line.append_after_offset(line.offset_to(pos), index);
        self.cursor.line = line_index;
    }
    pub fn line_insert(
        &mut self,
        pos: usize,
        index: Child,
    )
    {
        let mut sorted_lines = self
            .wall
            .iter()
            .enumerate()
            .map(|(i, lb)| (lb.pos, i))
            .sorted_by_key(|&(pos, _)| Reverse(pos))
            .map(|(_, i)| i)
            .collect_vec()
            .into_iter();

        sorted_lines
            .find_map(|line_index| {
                let line = self.get_line_mut(line_index);

                let index_width = index.width();
                let end_pos = line.end_pos();

                match pos.cmp(&end_pos)
                {
                    Ordering::Equal => Some(line_index),
                    Ordering::Less => None,
                    Ordering::Greater => panic!(
                        "should never happen {} > {}",
                        pos, end_pos,
                    ),
                }
            })
            .map(|line_index| self.append_at_line(line_index, pos, index))
            .unwrap_or_else(|| self.create_and_append_line(pos, index))
    }
    // pick current line, append index or create new line, advance cursor accordingly
    pub fn append_child(
        &mut self,
        pos: usize,
        index: Child,
    )
    {
        self.line_insert(pos, index);
    }
    pub fn close(mut self) -> Vec<Vec<PartitionCell>>
    {
        let end_pos = self.get_current_line().end_pos();
        self.wall
            .into_iter()
            .map(|line| line.close(end_pos))
            .collect()
    }
}

#[derive(Debug, IntoIterator)]
pub struct PartitionContainer
{
    wall: Vec<Vec<PartitionCell>>,
}
impl PartitionContainer
{
    pub fn from_child_list(list: impl IntoIterator<Item=(usize, Child)>) -> Self
    {
        let vec = list.into_iter()
            .sorted_by_key(|&(o, _)| o)
            .collect_vec();
        println!("{:#?}", vec.iter().map(|&(p, _)| p).collect_vec());

        vec.iter().tuple_windows().for_each(|((prev,_), (pos, _))|
            assert!(prev < pos, "{} < {}", prev, pos,)
        );
        let mut iter = vec.into_iter();
        let first = iter.next().unwrap();
        assert_eq!(first.0, 0);
        let mut builder = PartitionBuilder::new(first.0, first.1);
        while let Some((pos, index)) = iter.next()
        {
            builder.append_child(pos, index);
        }
        Self {
            wall: builder.close(),
        }
    }
}
