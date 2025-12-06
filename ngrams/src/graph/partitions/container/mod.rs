mod builder;

use builder::PartitionBuilder;
use context_trace::{
    graph::{
        vertex::{
            token::Token,
            data::{
                VertexData,
                VertexDataBuilder,
            },
            has_vertex_index::{
                HasVertexIndex,
                ToToken,
            },
            has_vertex_key::HasVertexKey,
            has_vertex_data::HasVertexData,
            pattern::Pattern,
            wide::Wide,
            VertexIndex,
            ChildPatterns,
        },
        Hypergraph,
    },
    HashMap,
    HashSet,
    VertexSet,
};
use derive_more::{
    Deref,
    DerefMut,
    IntoIterator,
};
use itertools::Itertools;
use pretty_assertions::assert_matches;
use range_ext::intersect::Intersect;
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
    traversal::direction::{
        TopDown,
        TraversalDirection,
    },
    utils::cover::ChildCover,
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
pub enum PartitionCell {
    ChildIndex(Token),
    GapSize(NonZeroUsize),
}
impl PartitionCell {
    pub fn width(&self) -> usize {
        match self {
            Self::ChildIndex(c) => c.width().0,
            Self::GapSize(o) => o.get(),
        }
    }
}
#[derive(Debug, IntoIterator, Deref)]
pub struct PartitionContainer {
    wall: Vec<Vec<Token>>,
}
impl PartitionContainer {
    pub fn from_ngram(
        ctx: &PartitionsCtx<'_>,
        ngram: NGramId,
    ) -> Self {
        // find all largest children
        let tree = ChildCover::from_key(ctx, ngram.vertex_key());

        assert!(match ngram.width().0 {
            1 => tree.is_empty(),
            _ => !tree.is_empty(),
        });

        // build container with gaps
        //let next = tree.iter().map(|(_, c)| c.vertex_index()).collect();
        let ctx = NodePartitionCtx::new(ngram, ctx);
        Self::from_child_list(&ctx, tree)
    }
    pub fn from_child_list(
        ctx: &NodePartitionCtx,
        list: impl IntoIterator<Item = (usize, NGramId)>,
    ) -> Self {
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
        for (pos, key) in child_iter {
            let index = ctx
                .vocab()
                .containment
                .expect_index_for_key(&key.vertex_key());
            builder.append_child(pos, Token::new(index, key.width()));
        }
        Self {
            wall: builder.close(),
        }
    }
}
impl Display for PartitionContainer {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        for line in &self.wall {
            for cell in line {
                let (t, s) = ("ch", cell.width());
                write!(f, "{}({})", t, s);
            }
            writeln!(f);
        }
        Ok(())
    }
}
