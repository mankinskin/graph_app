use std::sync::{
    Arc,
    RwLock,
};

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
    vocabulary::{
        entry::{
            HasVertexEntries,
            VocabEntry,
        },
        NGramId,
        Vocabulary,
    },
    Corpus,
};
use context_trace::{
    graph::vertex::{
        data::VertexDataBuilder,
        has_vertex_index::{
            HasVertexIndex,
            ToToken,
        },
        has_vertex_key::HasVertexKey,
        location::child::ChildLocation,
        pattern::id::PatternId,
        token::Token,
        wide::Wide,
        ChildPatterns,
    },
    HashMap,
};

use super::{
    Status,
    StatusHandle,
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
pub(crate) struct TextLocation {
    pub(crate) texti: usize,
    pub(crate) x: usize,
}

#[derive(Debug, From)]
pub(crate) struct CorpusCtx<'a> {
    pub(crate) corpus: &'a Corpus,
    pub(crate) status: &'a mut StatusHandle,
}
#[derive(Debug, Clone, Copy, From, Deref)]
pub(crate) struct TextLevelCtx<'a> {
    #[deref]
    pub(crate) corpus_ctx: &'a CorpusCtx<'a>,
    pub(crate) texti: usize,
    pub(crate) text: &'a String,
    pub(crate) n: usize,
}

impl TextLevelCtx<'_> {
    pub(crate) fn on_nlevel(
        &self,
        vocab: &mut Vocabulary,
    ) {
        let N: usize = self.text.len();
        self.text
            .chars()
            .ngrams(self.n)
            .enumerate()
            .for_each(|(xi, ngrami)| {
                NGramFrequencyCtx {
                    level_ctx: *self,
                    ngram: &String::from_iter(ngrami),
                    occurrence: TextLocation::new(self.texti, xi),
                }
                .on_ngram(vocab)
            })
    }
}

#[derive(Debug, Clone, Copy, From, Deref)]
pub(crate) struct NGramFrequencyCtx<'a> {
    #[deref]
    pub(crate) level_ctx: TextLevelCtx<'a>,
    pub(crate) ngram: &'a String,
    pub(crate) occurrence: TextLocation,
}

impl NGramFrequencyCtx<'_> {
    pub(crate) fn on_ngram(
        &self,
        vocab: &mut Vocabulary,
    ) {
        *self.status.steps_mut() += 1;
        if let Some(ctx) = vocab.get_vertex_mut(self.ngram) {
            ctx.entry.occurrences.insert(self.occurrence);
        } else {
            self.on_first_ngram(vocab)
        }
    }
    pub(crate) fn on_first_ngram(
        &self,
        vocab: &mut Vocabulary,
    ) {
        let children = self.find_children(vocab);

        let entry = VocabEntry {
            occurrences: FromIterator::from_iter([self.occurrence]),
            ngram: self.ngram.clone(),
        };
        let builder = if self.n != 1 {
            VertexDataBuilder::default()
                .width(self.n)
                .children(children.clone())
        } else {
            VertexDataBuilder::default().width(self.n)
        };
        let data = vocab.containment.finish_vertex_builder(builder);
        let id = NGramId::new(data.vertex_key(), self.n);
        vocab.ids.insert(self.ngram.clone(), id);
        vocab.entries.insert(id.vertex_key(), entry);
        if self.n == 1 {
            // Store the character as an atom for width=1 vertices
            let atom = self.ngram.chars().next().unwrap().into();
            vocab.containment.insert_atom_data(atom, data);
            vocab.leaves.insert(id);
        } else {
            vocab.containment.insert_vertex_data(data);
        }
        if self.n == self.level_ctx.text.len() {
            vocab.roots.insert(id);
        }
        for (pid, pat) in children {
            let child = Token::new(
                vocab.containment.expect_index_for_key(&id),
                id.width(),
            );
            let pat_vec: Vec<Token> = pat.into();
            vocab
                .containment
                .add_parents_to_pattern_nodes(pat_vec, child, pid);
        }
    }
    pub(crate) fn find_children(
        &self,
        vocab: &mut Vocabulary,
    ) -> ChildPatterns {
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
                            Token::new(
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
