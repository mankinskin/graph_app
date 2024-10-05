use derive_more::{
    Deref,
    From,
};
use derive_new::new;
use itertools::Itertools;
use ngram::NGram;
use serde::{
    Deserialize,
    Serialize,
};

use crate::graph::{
    Corpus,
    vocabulary::{
        entry::{
            HasVertexEntries,
            VocabEntry,
        }, NGramId, Vocabulary,
    }
};
use seqraph::{graph::vertex::{
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
}, HashMap};

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

#[derive(Debug, Clone, Copy, From)]
pub struct CorpusCtx<'a> {
    pub corpus: &'a Corpus,
}
impl CorpusCtx<'_>
{
    pub fn on_corpus(
        &self,
        vocab: &mut Vocabulary,
    )
    {
        let N: usize = self.corpus.iter().map(|t| t.len()).max().unwrap();
        Itertools::cartesian_product(
            (1..=N),
            self.corpus.iter().enumerate(),
        )
        .for_each(|(n, (i, text))|
            TextLevelCtx {
               texti: i,
               text,
               n,
            }.on_nlevel(vocab)
        )
    }
}
#[derive(Debug, Clone, Copy, From, Hash, Eq, PartialEq)]
pub struct TextLevelCtx<'a>
{
    pub texti: usize,
    pub text: &'a String,
    pub n: usize,
}

impl TextLevelCtx<'_>
{
    pub fn on_nlevel(
        &self,
        vocab: &mut Vocabulary,
    )
    {
        let N: usize = self.text.len();
        self.text.chars()
            .ngrams(self.n)
            .enumerate()
            .for_each(|(xi, ngrami)|
                NGramFrequencyCtx {
                    level_ctx: *self,
                    ngram: &String::from_iter(ngrami),
                    occurrence: TextLocation::new(self.texti, xi),
                }
                .on_ngram(vocab)
            )
    }
}

#[derive(Debug, Clone, Copy, From, Hash, Eq, PartialEq, Deref)]
pub struct NGramFrequencyCtx<'a>
{
    #[deref]
    pub level_ctx: TextLevelCtx<'a>,
    pub ngram: &'a String,
    pub occurrence: TextLocation,
}

impl NGramFrequencyCtx<'_>
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
    pub fn find_children(
        &self,
        vocab: &mut Vocabulary,
    ) -> ChildPatterns
    {
        /// find direct children
        let ngram = self.ngram.clone();
        let n = ngram.len() - 1;
        ngram
            .chars()
            .ngrams(n)
            .enumerate()
            // limit number of child patterns to 1 for 2-grams
            .take_while(|(i, _)| n != 1 || *i < 1)
            .map(|(i, ni)| {
                let f = ngram.get(0..i).filter(|s| !s.is_empty());
                let x = String::from_iter(ni);
                let b = ngram
                    .get((i + n)..ngram.len())
                    .filter(|s| !s.is_empty());
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
