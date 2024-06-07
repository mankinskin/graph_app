use derive_more::{
    Deref,
    From,
};
use derive_new::new;
use ngram::NGram;
use serde::{
    Deserialize,
    Serialize,
};

use seqraph::vertex::{
    indexed::{
        AsChild,
        Indexed,
    },
    ChildPatterns,
    VertexDataBuilder,
};

use crate::graph::vocabulary::{
    entry::{
        VocabEntry,
        IndexVocab,
    },
    NGramId,
    Vocabulary,
};

#[derive(
    Debug,
    Clone,
    Copy,
    From,
    new,
    Default,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub struct TextLocation
{
    pub texti: usize,
    pub x: usize,
}

#[derive(Debug, Clone, Copy, From, new, Hash, Eq, PartialEq)]
pub struct TextLevelCtx<'a>
{
    texti: usize,
    text: &'a String,
    n: usize,
}

impl<'a> TextLevelCtx<'a>
{
    pub fn on_nlevel(
        &self,
        vocab: &mut Vocabulary,
    )
    {
        let N: usize = self.text.len();
        let ngrams = self.text.chars().ngrams(self.n);
        ngrams.enumerate().for_each(|(xi, ni)| {
            let ngram = String::from_iter(ni);

            NGramFrequencyCtx::new(
                *self,
                &ngram,
                TextLocation::new(self.texti, xi),
            )
            .on_ngram(vocab);
        });
    }
}

#[derive(Debug, Clone, Copy, From, new, Hash, Eq, PartialEq, Deref)]
pub struct NGramFrequencyCtx<'a>
{
    #[deref]
    level_ctx: TextLevelCtx<'a>,
    ngram: &'a String,
    occurrence: TextLocation,
}

impl<'a> NGramFrequencyCtx<'a>
{
    pub fn on_ngram(
        &self,
        vocab: &mut Vocabulary,
    )
    {
        if let Some(ctx) = vocab.get_mut(self.ngram)
        {
            ctx.entry.occurrences.insert(self.occurrence);
        }
        else
        {
            self.on_first_ngram(vocab)
        }
    }
    pub fn on_first_ngram(
        &self,
        vocab: &mut Vocabulary,
    )
    {
        let id = NGramId::new(vocab.len(), self.n);
        let children = self.find_children(vocab);

        if self.n == 1 || !children.is_empty()
        {
            let entry = VocabEntry {
                occurrences: FromIterator::from_iter([self.occurrence]),
                ngram: self.ngram.clone(),
            };
            if self.n == 1
            {
                vocab.leaves.insert(id.vertex_index());
            }
            if self.n == self.level_ctx.text.len()
            {
                vocab.roots.insert(id.vertex_index());
            }
            vocab.ids.insert(self.ngram.clone(), id);
            vocab.entries.insert(id.vertex_index(), entry);
            let data = VertexDataBuilder::default()
                .index(id.id)
                .width(self.n.into())
                .children(children.clone())
                .build()
                .unwrap();
            vocab.graph.insert_vertex(data);
            for (pid, pat) in children
            {
                vocab.graph.add_parents_to_pattern_nodes(
                    pat,
                    id.as_child(),
                    pid,
                );
            }
        }
    }
    pub fn find_children(
        &self,
        vocab: &mut Vocabulary,
    ) -> ChildPatterns
    {
        /// find direct children
        let n = self.ngram.len() - 1;
        let ngram = self.ngram.clone();

        ngram
            .chars()
            .ngrams(n)
            .enumerate()
            .map(|(i, ni)| {
                let f = ngram.get(0..i).filter(|s| !s.is_empty());
                let x = String::from_iter(ni);
                let b =
                    ngram.get((i + n)..ngram.len()).filter(|s| !s.is_empty());
                let pid = vocab.graph.next_pattern_id();

                (
                    pid,
                    [f, Some(&x), b]
                        .into_iter()
                        .flatten()
                        .map(|s| {
                            let id = vocab.ids.get(s).unwrap();
                            let ctx = vocab.get(id).unwrap();
                            id.as_child()
                        })
                        .collect(),
                )
                //if entry.needs_node() {
                //if !children.covers_range(i..i+n, vocab) {
                //}
                //}
            })
            .collect()
        //(1..topn).rev()
        //    .flat_map(|n|
        //).collect()
    }
}
