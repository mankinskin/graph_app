pub mod output;

use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

use crate::{
    graph::vertex::pattern::id::PatternId,
    interval::PatternSplitPos,
    traversal::cache::entry::position::SubSplitLocation,
    HashMap,
};

use super::cache::position::SplitPositionCache;

use derive_more::derive::Deref;
use derive_new::new;
use itertools::Itertools;

use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            data::VertexData,
            location::SubLocation,
            wide::Wide,
        },
    },
    path::mutators::move_path::key::TokenPosition,
    traversal::{
        cache::entry::{
            position::Offset,
            vertex::VertexCache,
        },
        split::{
            position_splits,
            vertex::output::{
                NodeSplitOutput,
                NodeType,
                RootMode,
            },
        },
        traversable::Traversable,
    },
    HashSet,
};

#[derive(Debug, Clone, Copy)]
pub struct PosSplitContext<'a> {
    pub pos: &'a NonZeroUsize,
    pub split: &'a SplitPositionCache,
}

impl ToVertexSplits for PosSplitContext<'_> {
    fn to_vertex_splits(self) -> VertexSplits {
        VertexSplits {
            pos: *self.pos,
            splits: self.split.pattern_splits.clone(),
        }
    }
}

impl<'a, N: Borrow<(&'a NonZeroUsize, &'a SplitPositionCache)>> From<N> for PosSplitContext<'a> {
    fn from(item: N) -> Self {
        let (pos, split) = item.borrow();
        Self { pos, split }
    }
}
#[derive(Debug, Clone)]
pub struct VertexSplits {
    pub pos: NonZeroUsize,
    pub splits: PatternSplitPositions,
}

pub type PatternSplitPositions = HashMap<PatternId, PatternSplitPos>;

pub trait ToVertexSplits: Clone {
    fn to_vertex_splits(self) -> VertexSplits;
}

impl ToVertexSplits for VertexSplits {
    fn to_vertex_splits(self) -> VertexSplits {
        self
    }
}

impl ToVertexSplits for &VertexSplits {
    fn to_vertex_splits(self) -> VertexSplits {
        self.clone()
    }
}

impl<'a, N: Borrow<NonZeroUsize> + Clone, S: Borrow<SplitPositionCache> + Clone> ToVertexSplits
    for (N, S)
{
    fn to_vertex_splits(self) -> VertexSplits {
        VertexSplits::from(self)
    }
}
impl<'a, N: Borrow<NonZeroUsize>, S: Borrow<SplitPositionCache>> From<(N, S)> for VertexSplits {
    fn from(item: (N, S)) -> VertexSplits {
        VertexSplits {
            pos: *item.0.borrow(),
            splits: item.1.borrow().pattern_splits.clone(),
        }
    }
}

pub trait ToVertexSplitPos {
    fn to_vertex_split_pos(self) -> PatternSplitPositions;
}

impl ToVertexSplitPos for PatternSplitPositions {
    fn to_vertex_split_pos(self) -> PatternSplitPositions {
        self
    }
}

impl ToVertexSplitPos for Vec<SubSplitLocation> {
    fn to_vertex_split_pos(self) -> PatternSplitPositions {
        self.into_iter()
            .map(|loc| {
                (
                    loc.location.pattern_id,
                    PatternSplitPos {
                        inner_offset: loc.inner_offset,
                        sub_index: loc.location.sub_index,
                    },
                )
            })
            .collect()
    }
}

impl ToVertexSplitPos for VertexSplits {
    fn to_vertex_split_pos(self) -> PatternSplitPositions {
        self.splits
    }
}

#[derive(Debug, Copy, Clone, Deref, new)]
pub struct VertexSplitContext<'a> {
    pub cache: &'a VertexCache,
}
impl VertexSplitContext<'_> {
    pub fn global_splits<N: NodeType>(
        &self,
        end_pos: TokenPosition,
        node: &VertexData,
    ) -> N::GlobalSplitOutput {
        let mut output = N::GlobalSplitOutput::default();
        let (mut front, mut back) = (false, false);
        for (inner_width, cache) in &self.bottom_up {
            for location in cache.edges.bottom.values() {
                let child = node.expect_child_at(location);
                let inner_offset = Offset::new(child.width() - inner_width.0);
                let bottom = SubSplitLocation {
                    location: *location,
                    inner_offset,
                };
                let offset = node.expect_child_offset(location);
                if let Some(parent_offset) = inner_offset
                    .and_then(|o| o.checked_add(offset))
                    .or(NonZeroUsize::new(offset))
                {
                    output
                        .splits_mut()
                        .entry(parent_offset)
                        .and_modify(|e: &mut Vec<_>| e.push(bottom.clone()))
                        .or_insert_with(|| vec![bottom]);
                    front = true;
                } else {
                    break;
                }
            }
        }
        for (pretext_pos, cache) in &self.top_down {
            let inner_offset = Offset::new(end_pos.0 - pretext_pos.0).unwrap();
            for location in cache.edges.bottom.values() {
                let child = node.expect_child_at(location);
                let inner_offset = Offset::new(inner_offset.get() % child.width());
                let location = SubLocation {
                    sub_index: location.sub_index + inner_offset.is_none() as usize,
                    pattern_id: location.pattern_id,
                };
                let bottom = SubSplitLocation {
                    location,
                    inner_offset,
                };
                let offset = node.expect_child_offset(&location);
                let parent_offset = inner_offset
                    .map(|o| o.checked_add(offset).unwrap())
                    .unwrap_or_else(|| NonZeroUsize::new(offset).unwrap());
                if parent_offset.get() < node.width {
                    if let Some(e) = output.splits_mut().get_mut(&parent_offset) {
                        e.push(bottom)
                    } else {
                        output.splits_mut().insert(parent_offset, vec![bottom]);
                    }
                    back = true;
                }
            }
        }
        match (front, back) {
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
    ) -> N::CompleteSplitOutput {
        let graph = trav.graph();

        let node = graph.expect_vertex(self.index);

        let output = self.global_splits::<N>(end_pos, node);

        N::map(output, |global_splits| {
            global_splits
                .into_iter()
                .map(|(parent_offset, mut locs)| {
                    if locs.len() < node.children.len() {
                        let pids: HashSet<_> = locs.iter().map(|l| l.location.pattern_id).collect();
                        let missing = node
                            .children
                            .iter()
                            .filter(|(pid, _)| !pids.contains(pid))
                            .collect_vec();
                        let new_splits = position_splits(missing, parent_offset).splits;
                        locs.extend(new_splits.into_iter().map(|(pid, loc)| SubSplitLocation {
                            location: SubLocation::new(pid, loc.sub_index),
                            inner_offset: loc.inner_offset,
                        }))
                    }
                    (
                        parent_offset,
                        locs.into_iter()
                            .map(|sub| {
                                if sub.inner_offset.is_some()
                                    || node.children[&sub.location.pattern_id].len() > 2
                                {
                                    // can't be clean
                                    Ok(sub)
                                } else {
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
