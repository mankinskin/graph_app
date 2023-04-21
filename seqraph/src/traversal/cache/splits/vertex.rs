use crate::*;

#[derive(Debug, Default)]
pub struct SplitVertexCache {
    pub positions: BTreeMap<NonZeroUsize, SplitPositionCache>,
}
impl SplitVertexCache {
    pub fn new(pos: NonZeroUsize, entry: SplitPositionCache) -> Self {
        Self {
            positions: BTreeMap::from_iter([
                (
                    pos,
                    entry,
                )
            ]),
        }
    }
    pub fn pos_mut(&mut self, pos: NonZeroUsize) -> &mut SplitPositionCache {
        self.positions.entry(pos)
            .or_default()
    }
}
#[derive(Debug, Clone)]
pub struct PatternSplitPos {
    pub inner_offset: Option<NonZeroUsize>,
    pub sub_index: usize,
}
pub type PatternSubSplits = HashMap<PatternId, PatternSplitPos>;
#[derive(Debug, Default)]
pub struct SplitPositionCache {
    pub top: HashSet<SplitKey>,
    pub pattern_splits: PatternSubSplits,
    pub final_split: Option<FinalSplit>,
}

impl From<SubSplitLocation> for (PatternId, PatternSplitPos) {
    fn from(sub: SubSplitLocation) -> Self {
        (
            sub.location.pattern_id,
            PatternSplitPos {
                inner_offset: sub.inner_offset,
                sub_index: sub.location.sub_index,
            }
        )
    }
}
impl SplitPositionCache {
    pub fn root(subs: Vec<SubSplitLocation>) -> Self {
        Self {
            top: HashSet::default(),
            pattern_splits: subs.into_iter().map(Into::into).collect(),
            final_split: None,
        }
    }
    pub fn new(prev: SplitKey, subs: Vec<SubSplitLocation>) -> Self {
        Self {
            top: HashSet::from_iter([prev]),
            pattern_splits: subs.into_iter().map(Into::into).collect(),
            final_split: None,
        }
    }
    pub fn find_clean_split(&self) -> Option<SubLocation> {
        self.pattern_splits.iter().find_map(|(pid, s)|
            s.inner_offset.is_none().then_some(
                SubLocation {
                    pattern_id: *pid,
                    sub_index: s.sub_index,
                }
            )
        )
    }
    //pub fn add_location_split(&mut self, location: SubLocation, split: Split) {
    //    self.pattern_splits.insert(location, split);
    //}
    //pub fn join_splits(&mut self, indexer: &mut Indexer, key: &SplitKey) -> Split {
    //    let (l, r): (Vec<_>, Vec<_>) = self.pattern_splits
    //        .drain()
    //        .map(|(_, s)| (s.left, s.right))
    //        .unzip();
    //    // todo detect existing splits
    //    let mut graph = indexer.graph_mut();
    //    let lc = graph.insert_patterns(l);
    //    let rc = graph.insert_patterns(r);
    //    graph.add_pattern_with_update(&key.index, vec![lc, rc]);
    //    let split = Split {
    //        left: vec![lc],
    //        right: vec![rc],
    //    };
    //    self.final_split = Some(split.clone());
    //    split
    //}
}
//impl From<Split> for SplitPositionCache {
//    fn from(split: Split) -> Self {
//        Self {
//            pattern_splits: Default::default(),
//            final_split: Some(split),
//        }
//    }
//}
