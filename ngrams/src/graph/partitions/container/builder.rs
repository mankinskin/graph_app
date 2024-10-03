use crate::graph::{
    partitions::{
        container::PartitionCell,
        NodePartitionCtx,
    },
    vocabulary::NGramId,
};
use derive_more::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use ngram::NGram;
use seqraph::graph::{
    vertex::{
        child::Child,
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        wide::Wide,
    },
    getters::vertex::VertexSet,
};
use std::{
    cmp::{
        Ordering,
        Reverse,
    },
    num::NonZeroUsize,
};

#[derive(Debug, Deref, DerefMut)]
pub struct PartitionLineBuilder<'a, 'b>
{
    pos: usize,
    ctx: &'a NodePartitionCtx<'a, 'b>,
    #[deref]
    #[deref_mut]
    line: Vec<Child>,
}
impl<'a, 'b> PartitionLineBuilder<'a, 'b>
{
    pub fn new(ctx: &'a NodePartitionCtx<'a, 'b>) -> Self
    {
        Self {
            ctx,
            pos: Default::default(),
            line: Default::default(),
        }
    }
    fn push_cell(
        &mut self,
        cell: Child,
    )
    {
        if let Some(last) = self.line.last()
        {
            self.pos += last.width();
        }
        self.line.push(cell);
    }
    pub fn append(
        &mut self,
        index: Child,
    )
    {
        self.push_cell(index);
    }
    pub fn skip(
        &mut self,
        offset: NonZeroUsize,
    )
    {
        let end_pos = self.end_pos();
        let index = self.ctx.vocab.containment.get_vertex_subrange(
            self.ctx.root.vertex_key(),
            end_pos..(end_pos + offset.get()),
        );
        self.push_cell(index);
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
        NonZeroUsize::new(pos - self.end_pos())
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
        self.pos + self.line.last().map(|cell|
            cell.width()
        ).unwrap_or_default()
    }
    pub fn close(
        mut self,
        end_pos: usize,
    ) -> Vec<Child>
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
    pub fn get_current_line<'a, 'b>(
        &self,
        builder: &'a mut PartitionBuilder<'a, 'b>,
    ) -> &'a mut PartitionLineBuilder<'a, 'b>
    {
        builder.get_line_mut(self.line)
    }
}

#[derive(Debug)]
pub struct PartitionBuilder<'a, 'b>
{
    ctx: &'a NodePartitionCtx<'a, 'b>,
    cursor: PartitionCursor,
    wall: Vec<PartitionLineBuilder<'a, 'b>>,
}

impl<'a, 'b> PartitionBuilder<'a, 'b>
{
    pub fn new(
        ctx: &'a NodePartitionCtx<'a, 'b>,
        offset: usize,
        first: NGramId,
    ) -> Self
    {
        let mut builder = Self {
            ctx,
            cursor: PartitionCursor { line: 0 },
            wall: Default::default(),
        };
        let index = ctx.vocab.containment.expect_index_for_key(&first.vertex_key());
        builder.append_child(offset, Child::new(index, first.width()));
        builder
    }
    pub fn create_line(
        &mut self,
        pos: usize,
        index: Child,
    )
    {
        let mut line = PartitionLineBuilder::new(self.ctx);
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
        let mut line = PartitionLineBuilder::new(self.ctx);
        line.append_after_offset(NonZeroUsize::new(pos), index);
        self.cursor.line = self.wall.len();
        self.wall.push(line);
    }
    pub fn get_line_mut(
        &mut self,
        index: usize,
    ) -> &mut PartitionLineBuilder<'a, 'b>
    {
        self.wall.get_mut(index).expect("Invalid PartitionCursor")
    }
    pub fn get_current_line(&mut self) -> &mut PartitionLineBuilder<'a, 'b>
    {
        self.get_line_mut(self.cursor.line)
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
    // pick current line, append index or create new line, advance cursor accordingly
    pub fn append_child(
        &mut self,
        pos: usize,
        index: Child,
    )
    {
        //println!("Find line to insert {}..{}", pos, pos + index.width());
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
            .find(|line_index| {
                let line = self.get_line_mut(*line_index);

                let index_width = index.width();
                let end_pos = line.end_pos();
                //println!("{}", end_pos);

                match end_pos.cmp(&pos)
                {
                    Ordering::Equal | Ordering::Less => true,
                    Ordering::Greater => false,
                }
            })
            .map(|line_index| {
                //println!("Append {}", pos);
                self.append_at_line(line_index, pos, index)
            })
            .unwrap_or_else(|| {
                //println!("Create new {}", pos);
                self.create_and_append_line(pos, index)
            })
    }
    pub fn close(mut self) -> Vec<Vec<Child>>
    {
        let end_pos = self.get_current_line().end_pos();
        self.wall
            .into_iter()
            .map(|line| line.close(end_pos))
            .collect()
    }
}
