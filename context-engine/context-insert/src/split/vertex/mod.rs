pub mod output;

use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use super::cache::position::SplitPositionCache;

use crate::split::{
    position_splits,
    vertex::output::{
        NodeSplitOutput,
        NodeType,
        RootMode,
    },
};
use context_search::{
    HashMap,
    HashSet,
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            data::VertexData,
            location::SubLocation,
            pattern::id::PatternId,
            wide::Wide,
        },
    },
    path::mutators::move_path::key::TokenPosition,
    trace::child::ChildTracePos,
    traversal::{
        cache::entry::{
            position::{
                Offset,
                SubSplitLocation,
            },
            vertex::VertexCache,
        },
        traversable::Traversable,
    },
};
use derive_more::derive::Deref;
use derive_new::new;
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub struct PosSplitContext<'a>
{
    pub pos: &'a NonZeroUsize,
    pub split: &'a SplitPositionCache,
}

impl ToVertexSplits for PosSplitContext<'_>
{
    fn to_vertex_splits(self) -> VertexSplits
    {
        VertexSplits {
            pos: *self.pos,
            splits: self.split.pattern_splits.clone(),
        }
    }
}

impl<'a, N: Borrow<(&'a NonZeroUsize, &'a SplitPositionCache)>> From<N>
    for PosSplitContext<'a>
{
    fn from(item: N) -> Self
    {
        let (pos, split) = item.borrow();
        Self { pos, split }
    }
}
#[derive(Debug, Clone)]
pub struct VertexSplits
{
    pub pos: NonZeroUsize,
    pub splits: ChildTracePositions,
}

pub type ChildTracePositions = HashMap<PatternId, ChildTracePos>;

pub trait ToVertexSplits: Clone
{
    fn to_vertex_splits(self) -> VertexSplits;
}

impl ToVertexSplits for VertexSplits
{
    fn to_vertex_splits(self) -> VertexSplits
    {
        self
    }
}

impl ToVertexSplits for &VertexSplits
{
    fn to_vertex_splits(self) -> VertexSplits
    {
        self.clone()
    }
}

impl<'a, N: Borrow<NonZeroUsize> + Clone, S: Borrow<SplitPositionCache> + Clone>
    ToVertexSplits for (N, S)
{
    fn to_vertex_splits(self) -> VertexSplits
    {
        VertexSplits::from(self)
    }
}
impl<'a, N: Borrow<NonZeroUsize>, S: Borrow<SplitPositionCache>> From<(N, S)>
    for VertexSplits
{
    fn from(item: (N, S)) -> VertexSplits
    {
        VertexSplits {
            pos: *item.0.borrow(),
            splits: item.1.borrow().pattern_splits.clone(),
        }
    }
}

pub trait ToVertexSplitPos
{
    fn to_vertex_split_pos(self) -> ChildTracePositions;
}

impl ToVertexSplitPos for ChildTracePositions
{
    fn to_vertex_split_pos(self) -> ChildTracePositions
    {
        self
    }
}

impl ToVertexSplitPos for Vec<SubSplitLocation>
{
    fn to_vertex_split_pos(self) -> ChildTracePositions
    {
        self.into_iter()
            .map(|loc| {
                (loc.location.pattern_id, ChildTracePos {
                    inner_offset: loc.inner_offset,
                    sub_index: loc.location.sub_index,
                })
            })
            .collect()
    }
}

impl ToVertexSplitPos for VertexSplits
{
    fn to_vertex_split_pos(self) -> ChildTracePositions
    {
        self.splits
    }
}

#[derive(Debug, Copy, Clone, Deref, new)]
pub struct VertexSplitContext<'a>
{
    pub cache: &'a VertexCache,
}
impl VertexSplitContext<'_>
{
    pub fn bottom_up_splits<N: NodeType>(
        &self,
        node: &VertexData,
        output: &mut N::GlobalSplitOutput,
    ) -> bool
    {
        let mut front = false;
        // uses inner width of sub split position to calculate node offset
        for (inner_width, pos_cache) in self.bottom_up.iter()
        {
            // bottom up incoming edge
            for location in pos_cache.edges.bottom.values()
            {
                // pattern location
                let child = node.expect_child_at(location);

                let inner_offset = Offset::new(child.width() - inner_width.0);
                let outer_offset = node.expect_child_offset(location);
                if let Some(node_offset) = inner_offset
                    .and_then(|o| o.checked_add(outer_offset))
                    .or(NonZeroUsize::new(outer_offset))
                {
                    let split_loc = SubSplitLocation {
                        location: *location,
                        inner_offset,
                    };
                    output
                        .splits_mut()
                        .entry(node_offset)
                        .and_modify(|e: &mut Vec<_>| e.push(split_loc.clone()))
                        .or_insert_with(|| vec![split_loc]);
                    front = true;
                }
                else
                {
                    break;
                }
            }
        }
        front
    }
    pub fn top_down_splits<N: NodeType>(
        &self,
        end_pos: TokenPosition,
        node: &VertexData,
        output: &mut N::GlobalSplitOutput,
    ) -> bool
    {
        let mut back = false;
        // uses end pos of sub split position to calculate node offset
        for (outer_offset, pos_cache) in self.top_down.iter()
        {
            // outer offset:
            let inner_offset = Offset::new(end_pos.0 - outer_offset.0).unwrap();
            for location in pos_cache.edges.bottom.values()
            {
                let child = node.expect_child_at(location);
                let inner_offset =
                    Offset::new(inner_offset.get() % child.width());
                let location = SubLocation {
                    sub_index: location.sub_index
                        + inner_offset.is_none() as usize,
                    pattern_id: location.pattern_id,
                };

                let offset = node.expect_child_offset(&location);
                let parent_offset = inner_offset
                    .map(|o| o.checked_add(offset).unwrap())
                    .unwrap_or_else(|| NonZeroUsize::new(offset).unwrap());

                if parent_offset.get() < node.width
                {
                    let bottom = SubSplitLocation {
                        location,
                        inner_offset,
                    };
                    if let Some(e) = output.splits_mut().get_mut(&parent_offset)
                    {
                        e.push(bottom)
                    }
                    else
                    {
                        output.splits_mut().insert(parent_offset, vec![bottom]);
                    }
                    back = true;
                }
            }
        }
        back
    }
    pub fn global_splits<N: NodeType>(
        &self,
        end_pos: TokenPosition,
        node: &VertexData,
    ) -> N::GlobalSplitOutput
    {
        let mut output = N::GlobalSplitOutput::default();
        let front = self.bottom_up_splits::<N>(node, &mut output);
        let back = self.top_down_splits::<N>(end_pos, node, &mut output);
        match (front, back)
        {
            (true, true) => output.set_root_mode(RootMode::Infix),
            (false, true) => output.set_root_mode(RootMode::Prefix),
            (true, false) => output.set_root_mode(RootMode::Postfix),
            (false, false) => unreachable!(),
        }
        output
    }
    pub fn complete_splits<Trav: Traversable, N: NodeType>(
        &self,
        trav: &Trav,
        end_pos: TokenPosition,
    ) -> N::CompleteSplitOutput
    {
        let graph = trav.graph();

        let node = graph.expect_vertex(self.index);

        let output = self.global_splits::<N>(end_pos, node);

        N::map(output, |global_splits| {
            global_splits
                .into_iter()
                .map(|(parent_offset, mut locs)| {
                    if locs.len() < node.children.len()
                    {
                        let pids: HashSet<_> = locs
                            .iter()
                            .map(|l| l.location.pattern_id)
                            .collect();
                        let missing = node
                            .children
                            .iter()
                            .filter(|(pid, _)| !pids.contains(pid))
                            .collect_vec();
                        let new_splits =
                            position_splits(missing, parent_offset).splits;
                        locs.extend(new_splits.into_iter().map(|(pid, loc)| {
                            SubSplitLocation {
                                location: SubLocation::new(pid, loc.sub_index),
                                inner_offset: loc.inner_offset,
                            }
                        }))
                    }
                    (
                        parent_offset,
                        locs.into_iter()
                            .map(|sub| {
                                if sub.inner_offset.is_some()
                                    || node.children[&sub.location.pattern_id]
                                        .len()
                                        > 2
                                {
                                    // can't be clean
                                    Ok(sub)
                                }
                                else
                                {
                                    // must be clean
                                    Err(sub.location)
                                }
                            })
                            .collect(),
                    )
                })
                .collect()
        })
    }
}
