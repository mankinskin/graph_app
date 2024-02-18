use crate::{shared::*, TextLocation};


#[derive(Debug, Deref)]
pub struct VocabEntry {
    //pub id: NGramId,
    pub occurrences: HashSet<TextLocation>,
    // positions of largest smaller ngrams
    //pub children: NodeChildren,
    #[deref]
    pub ngram: String,
} 

impl VocabEntry {
    pub fn count(&self) -> usize {
        self.occurrences.len()
    }
    //pub fn needs_node(&self) -> bool {
    //    self.len() == 1
    //        || self.children.has_overlaps()
    //}
}

//pub type EntryMap = HashMap<NGramId, VocabEntry>;

//#[derive(Debug, Deref, DerefMut, From, Default)]
//pub struct NodeChildren<N: ChildEntry = NGramId>(pub HashMap<usize, N>);
//
//pub trait ChildEntry {}
//impl<T> ChildEntry for T {}

//impl NodeChildren<NGramId> {
//    pub fn covers_range(&self, range: Range<usize>, vocab: &Vocabulary) -> bool {
//        self.iter().any(|(&x, id)| {
//            let node = vocab.entries.get(&id).unwrap();
//            x <= range.start && range.end <= (x + node.len())
//        })
//    }
//    //pub fn has_overlaps(&self, vocab: &Vocabulary) -> bool {
//    //    self.map(|id| vocab.get(id).unwrap())
//    //        .has_overlaps()
//    //}
//    pub fn has_overlaps(&self) -> bool {
//        let mut last = 0;
//        self.iter().sorted_by_key(|(&k, _)| k).any(|(&x, n)| {
//            assert!(x <= last);
//            let r = x < last;
//            last = x + n.width;
//            r
//        })
//    }
//}
////impl<T: ChildEntry> NodeChildren<T> {
////    pub fn map<N: ChildEntry, F: FnMut(&T) -> N>(&self, mut f: F) -> NodeChildren<N> {
////        NodeChildren(
////            self.0.iter()
////                .map(|(i, x)| (*i, f(x)))
////                .collect()
////        )
////    }
////}
//impl<E: ChildEntry + Borrow<VocabEntry>> NodeChildren<E> {
//    pub fn num_partitions(&self) -> usize {
//        let mut lim: HashSet<usize> = Default::default();
//        self.iter().fold(1, |mut count, (x, n)| {
//            count += lim.contains(x) as usize;
//            lim.insert(x + n.borrow().len());
//            count
//        })
//    }
//}