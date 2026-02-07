use crate::graph::{
    partitions::{
        container::PartitionCell,
        NodePartitionCtx,
    },
    vocabulary::NGramId,
};
use context_trace::graph::{
    vertex::{
        token::Token,
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        wide::Wide,
        VertexIndex,
        data::VertexData,
        pattern::Pattern,
        ChildPatterns,
    },
};
use context_trace::VertexSet;
use derive_more::{
    Deref,
    DerefMut,
};
use itertools::Itertools;
use ngram::NGram;
use std::{
    cmp::{
        Ordering,
        Reverse,
    },
    num::NonZeroUsize,
};

#[derive(Debug, Deref, DerefMut)]
pub(crate) struct PartitionLineBuilder<'a, 'b> {
    pos: usize,
    ctx: &'a NodePartitionCtx<'a, 'b>,
    #[deref]
    #[deref_mut]
    line: Vec<Token>,
}
impl<'a, 'b> PartitionLineBuilder<'a, 'b> {
    pub(crate) fn new(ctx: &'a NodePartitionCtx<'a, 'b>) -> Self {
        Self {
            ctx,
            pos: Default::default(),
            line: Default::default(),
        }
    }
    fn push_cell(
        &mut self,
        cell: Token,
    ) {
        if let Some(last) = self.line.last() {
            self.pos += last.width().0;
        }
        self.line.push(cell);
    }
    pub(crate) fn append(
        &mut self,
        index: Token,
    ) {
        self.push_cell(index);
    }
    pub(crate) fn skip(
        &mut self,
        offset: NonZeroUsize,
    ) {
        let end_pos = self.end_pos();
        // Get the root data to use its token for get_vertex_subrange
        let root_data = self.ctx.vocab().containment.expect_vertex_data(self.ctx.root.vertex_key());
        let root_token = root_data.to_token();
        let index = self.ctx.vocab().containment.get_vertex_subrange(
            root_token,
            end_pos..(end_pos + offset.get()),
        );
        self.push_cell(index);
    }
    pub(crate) fn append_after_offset(
        &mut self,
        offset: Option<NonZeroUsize>,
        index: Token,
    ) {
        if let Some(non_zero) = offset {
            self.skip(non_zero);
        }
        self.append(index);
    }
    pub(crate) fn offset_to(
        &self,
        pos: usize,
    ) -> Option<NonZeroUsize> {
        NonZeroUsize::new(pos - self.end_pos())
    }
    pub(crate) fn skip_to(
        &mut self,
        end_pos: usize,
    ) {
        if let Some(o) = self.offset_to(end_pos) {
            self.skip(o);
        }
    }
    pub(crate) fn end_pos(&self) -> usize {
        self.pos
            + self
                .line
                .last()
                .map(|cell| cell.width().0)
                .unwrap_or_default()
    }
    pub(crate) fn close(
        mut self,
        end_pos: usize,
    ) -> Vec<Token> {
        self.skip_to(end_pos);
        self.line
    }
}

#[derive(Debug, Clone, Copy, Hash)]
struct PartitionCursor {
    pub(crate) line: usize,
}
impl PartitionCursor {
    pub(crate) fn get_current_line<'a, 'b>(
        &self,
        builder: &'a mut PartitionBuilder<'a, 'b>,
    ) -> &'a mut PartitionLineBuilder<'a, 'b> {
        builder.get_line_mut(self.line)
    }
}

#[derive(Debug)]
pub(crate) struct PartitionBuilder<'a, 'b> {
    ctx: &'a NodePartitionCtx<'a, 'b>,
    cursor: PartitionCursor,
    wall: Vec<PartitionLineBuilder<'a, 'b>>,
}

impl<'a, 'b> PartitionBuilder<'a, 'b> {
    pub(crate) fn new(
        ctx: &'a NodePartitionCtx<'a, 'b>,
        offset: usize,
        first: NGramId,
    ) -> Self {
        let mut builder = Self {
            ctx,
            cursor: PartitionCursor { line: 0 },
            wall: Default::default(),
        };
        let index = ctx
            .vocab()
            .containment
            .expect_index_for_key(&first.vertex_key());
        builder.append_child(offset, Token::new(index, first.width()));
        builder
    }
    pub(crate) fn create_line(
        &mut self,
        pos: usize,
        index: Token,
    ) {
        let mut line = PartitionLineBuilder::new(self.ctx);
        line.append_after_offset(NonZeroUsize::new(pos), index);
        self.cursor.line = self.wall.len();
        self.wall.push(line);
    }
    pub(crate) fn create_and_append_line(
        &mut self,
        pos: usize,
        index: Token,
    ) {
        let mut line = PartitionLineBuilder::new(self.ctx);
        line.append_after_offset(NonZeroUsize::new(pos), index);
        self.cursor.line = self.wall.len();
        self.wall.push(line);
    }
    pub(crate) fn get_line_mut(
        &mut self,
        index: usize,
    ) -> &mut PartitionLineBuilder<'a, 'b> {
        self.wall.get_mut(index).expect("Invalid PartitionCursor")
    }
    pub(crate) fn get_current_line(&mut self) -> &mut PartitionLineBuilder<'a, 'b> {
        self.get_line_mut(self.cursor.line)
    }
    pub(crate) fn append_at_line(
        &mut self,
        line_index: usize,
        pos: usize,
        index: Token,
    ) {
        let line = self.get_line_mut(line_index);
        line.append_after_offset(line.offset_to(pos), index);
        self.cursor.line = line_index;
    }
    // pick current line, append index or create new line, advance cursor accordingly
    pub(crate) fn append_child(
        &mut self,
        pos: usize,
        index: Token,
    ) {
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

                match end_pos.cmp(&pos) {
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
    pub(crate) fn close(mut self) -> Vec<Vec<Token>> {
        let end_pos = self.get_current_line().end_pos();
        self.wall
            .into_iter()
            .map(|line| line.close(end_pos))
            .collect()
    }
}
