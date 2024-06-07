use seqraph::{
    vertex::VertexIndex,
    HashSet,
};

use crate::graph::vocabulary::{
    ProcessStatus,
    Vocabulary,
};

mod frequency;
use frequency::FrequencyCtx;

mod wrappers;
use crate::graph::partitions::PartitionsCtx;
use wrappers::WrapperCtx;

#[derive(Debug)]
pub struct LabellingCtx
{
    pub labels: HashSet<VertexIndex>,
}
impl LabellingCtx
{
    pub fn new() -> Self
    {
        let mut ctx = Self {
            labels: Default::default(),
            //FromIterator::from_iter(
            //    vocab.leaves.iter().chain(
            //        vocab.roots.iter()
            //    )
            //    .cloned(),
            //),
        };
        ctx
    }
}
pub fn label_vocab(vocab: &mut Vocabulary) -> HashSet<VertexIndex>
{
    //let roots = texts.iter().map(|s| *vocab.ids.get(s).unwrap()).collect_vec();
    let mut ctx = LabellingCtx::new();
    if (vocab.status < ProcessStatus::Frequency)
    {
        println!("Frequency Pass");
        FrequencyCtx::from(&mut ctx).frequency_pass(vocab);
        vocab.status = ProcessStatus::Frequency;
        vocab.labels = ctx.labels.clone();
        //vocab.write_to_file(vocab.target_file_path());
    }
    else
    {
        println!("Frequency Pass already processed.");
        ctx.labels = vocab.labels.clone();
    }
    if (vocab.status < ProcessStatus::Wrappers)
    {
        println!("Wrapper Pass");
        WrapperCtx::from(&mut ctx).wrapping_pass(vocab);
        vocab.labels = ctx.labels.clone();
        vocab.status = ProcessStatus::Wrappers;
        vocab.write_to_file(vocab.target_file_path());
    }
    else
    {
        println!("Wrapper Pass already processed.");
        ctx.labels = vocab.labels.clone();
    }
    if (vocab.status < ProcessStatus::Partitions)
    {
        println!("Partition Pass");
        PartitionsCtx::from(&mut ctx).partitions_pass(vocab);
        vocab.labels = ctx.labels.clone();
        vocab.status = ProcessStatus::Partitions;
        //vocab.write_to_file(vocab.target_file_path());
    }
    else
    {
        println!("Partition Pass already processed.");
        ctx.labels = vocab.labels.clone();
    }
    //println!("{:#?}", vocab.labels);
    vocab.labels.clone()
}
