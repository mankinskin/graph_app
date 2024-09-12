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

use crate::graph::vocabulary::{
    entry::{
        HasVertexEntries,
        VocabEntry,
    },
    NGramId,
    Vocabulary,
};
use seqraph::graph::vertex::{
    child::Child,
    data::VertexDataBuilder,
    has_vertex_index::{
        HasVertexIndex,
        ToChild,
    },
    has_vertex_key::HasVertexKey,
    key::VertexKey,
    pattern::id::PatternId,
    token::Token,
    wide::Wide,
    ChildPatterns,
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
        if let Some(ctx) = vocab.get_vertex_mut(self.ngram)
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
        let children = self.find_children(vocab);

        if self.n == 1 || !children.is_empty()
        {
            let entry = VocabEntry {
                occurrences: FromIterator::from_iter([self.occurrence]),
                ngram: self.ngram.clone(),
            };
            let mut builder = VertexDataBuilder::default();
            builder.width(self.n);
            if self.n != 1
            {
                builder.children(children.clone());
            }
            let data = vocab.containment.finish_vertex_builder(builder);
            let id = NGramId::new(data.key, self.n);
            vocab.ids.insert(self.ngram.clone(), id);
            vocab.entries.insert(id.vertex_key(), entry);
            if self.n == 1
            {
                vocab.containment.insert_token_data(
                    Token::Element(self.ngram.chars().next().unwrap()),
                    data,
                );
                vocab.leaves.insert(id);
            }
            else
            {
                vocab.containment.insert_vertex_data(data);
            }
            if self.n == self.level_ctx.text.len()
            {
                vocab.roots.insert(id);
            }
            for (pid, pat) in children
            {
                let child = Child::new(
                    vocab.containment.expect_index_for_key(&id),
                    id.width(),
                );
                vocab
                    .containment
                    .add_parents_to_pattern_nodes(pat, child, pid);
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
                let pid = PatternId::default();

                (
                    pid,
                    [f, Some(&x), b]
                        .into_iter()
                        .flatten()
                        .map(|s| {
                            let id = vocab.ids.get(s).unwrap();
                            Child::new(
                                vocab.containment.expect_index_for_key(id),
                                id.width(),
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }
}
