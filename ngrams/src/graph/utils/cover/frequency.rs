use std::collections::VecDeque;

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use itertools::Itertools;

use crate::graph::{
    containment::TextLocation,
    labelling::LabellingCtx,
    traversal::{
        direction::{
            BottomUp,
            TopDown,
            TraversalDirection,
        },
        pass::RunResult,
    },
    vocabulary::{
        entry::{
            HasVertexEntries,
            VertexCtx,
        },
        NGramId,
        ProcessStatus,
        Vocabulary,
    },
};
use context_trace::{
    graph::vertex::{
        has_vertex_index::HasVertexIndex,
        has_vertex_key::HasVertexKey,
        location::child::{
            ChildLocation,
            HasSubIndex,
        },
        parent::HasPatternId,
        pattern::Pattern,
        token::Token,
        wide::Wide,
        VertexIndex,
    },
    HashSet,
};

#[derive(Debug, Default, Clone, Deref)]
pub(crate) struct FrequencyCover {
    #[deref]
    cover: HashSet<NGramId>,
}
impl FrequencyCover {
    pub(crate) fn from_entry(
        ctx: &LabellingCtx,
        entry: &VertexCtx,
    ) -> RunResult<Self> {
        let mut cover: HashSet<_> = Default::default();
        let mut occ_set: HashSet<_> = Default::default(); //entry.occurrences.clone();
        let mut queue: VecDeque<_> =
            FromIterator::from_iter(Self::next_parent_offsets(entry));
        while let Some((off, p)) = queue.pop_front() {
            ctx.check_cancelled()?;
            let pe = entry.vocab.get_vertex(&p).unwrap();
            let diff = Self::new_occurrences(ctx, off, &pe, &occ_set);
            if diff.is_empty() {
                queue.extend(
                    Self::next_parent_offsets(&pe)
                        .into_iter()
                        .map(|(o, p)| (o + off, p)),
                );
            } else {
                cover.insert(p);
                occ_set.extend(&diff);
            }
        }
        Ok(Self { cover })
    }
    fn next_parent_offsets(entry: &VertexCtx) -> Vec<(usize, NGramId)> {
        entry
            .data
            .parents()
            .iter()
            .flat_map(|(&id, p)| {
                p.pattern_indices().iter().map(move |ploc| {
                    (
                        entry.vocab.containment.expect_child_offset(
                            &ChildLocation::new(
                                Token::new(id, p.width()),
                                ploc.pattern_id(),
                                ploc.sub_index(),
                            ),
                        ),
                        NGramId::new(
                            entry.vocab.containment.expect_key_for_index(id),
                            p.width().0,
                        ),
                    )
                })
            })
            .collect_vec()
    }
    pub(crate) fn new_occurrences(
        ctx: &LabellingCtx,
        offset: usize,
        parent_entry: &VertexCtx,
        occ_set: &HashSet<TextLocation>,
    ) -> HashSet<TextLocation> {
        if ctx.labels().contains(&parent_entry.vertex_key()) {
            {
                let occ: HashSet<_> = parent_entry
                    .occurrences
                    .iter()
                    .map(|loc| TextLocation::new(loc.texti, loc.x + offset))
                    .collect();
                occ.difference(occ_set).copied().collect()
            }
        } else {
            Default::default()
        }
    }
}
