use derive_new::new;
use itertools::Itertools;
use ngram::NGram;
use pretty_assertions::assert_eq;

use seqraph::HashSet;

use crate::graph::vocabulary::Corpus;
use crate::graph::{
    vocabulary::{
        entry::HasVertexEntries,
        Vocabulary,
    },
};
use crate::graph::labelling::{LabellingCtx};

mod containment;
mod labelling;
mod partitions;
mod traversal;
mod vocabulary;
pub mod tests;

pub fn test_graph()
{
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let texts = corpus.into_iter().map(ToString::to_string).collect_vec();
    let corpus = Corpus::new("ottos_mops".to_owned(), texts);
    // graph of all containment edges between n and n+1
    let mut image = LabellingCtx::from_corpus(&corpus);

    {
        tests::TestCtx::new(&image.vocab, &corpus)
            .test_containment();
    }

    image.label_freq();

    {
        let ctx = tests::LabelTestCtx::new(
            tests::TestCtx::new(&image.vocab, &corpus),
            &image.labels,
        );
        ctx.test_roots();
        ctx.test_leaves();

        ctx.test_freq();
    }

    image.label_wrap();

    {
        let ctx = tests::LabelTestCtx::new(
            tests::TestCtx::new(&image.vocab, &corpus),
            &image.labels,
        );
        ctx.test_wrap();

    }

    image.label_part();

    {
        let ctx = tests::TestCtx::new(&image.vocab, &corpus);
        let ctx = tests::LabelTestCtx::new(ctx, &image.labels);
        ctx.test_part();
    }
}

