use std::{
    borrow::{
        Borrow,
        BorrowMut,
    },
    fmt::Debug,
    iter::FromIterator,
    num::NonZeroUsize,
};

use crate::{
    interval::partition::delta::PatternSubDeltas,
    split::vertex::{
        ChildTracePositions,
        ToVertexSplitPos,
    },
};
use context_trace::{
    HashSet,
    graph::vertex::{
        child::Child,
        location::SubLocation,
        wide::Wide,
    },
    trace::cache::entry::position::SubSplitLocation,
};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct PosKey {
    pub index: Child,
    pub pos: NonZeroUsize,
}

impl PosKey {
    pub fn new<P: TryInto<NonZeroUsize>>(
        index: Child,
        pos: P,
    ) -> Self
    where
        P::Error: Debug,
    {
        Self {
            index,
            pos: pos.try_into().unwrap(),
        }
    }
}

impl From<Child> for PosKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: NonZeroUsize::new(index.width()).unwrap(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SplitPositionCache {
    pub top: HashSet<PosKey>,
    pub pattern_splits: ChildTracePositions,
}

impl std::ops::Sub<PatternSubDeltas> for SplitPositionCache {
    type Output = Self;
    fn sub(
        mut self,
        rhs: PatternSubDeltas,
    ) -> Self::Output {
        self.pattern_splits
            .iter_mut()
            .for_each(|(pid, pos)| pos.sub_index -= rhs[pid]);
        self
    }
}

impl Borrow<ChildTracePositions> for SplitPositionCache {
    fn borrow(&self) -> &ChildTracePositions {
        &self.pattern_splits
    }
}

impl Borrow<ChildTracePositions> for &SplitPositionCache {
    fn borrow(&self) -> &ChildTracePositions {
        &self.pattern_splits
    }
}

impl BorrowMut<ChildTracePositions> for SplitPositionCache {
    fn borrow_mut(&mut self) -> &mut ChildTracePositions {
        &mut self.pattern_splits
    }
}

impl SplitPositionCache {
    pub fn root(subs: impl ToVertexSplitPos) -> Self {
        Self {
            top: HashSet::default(),
            pattern_splits: subs.to_vertex_split_pos(),
        }
    }
    pub fn new(
        prev: PosKey,
        subs: Vec<SubSplitLocation>,
    ) -> Self {
        Self {
            top: HashSet::from_iter(Some(prev)),
            pattern_splits: subs.into_iter().map(Into::into).collect(),
        }
    }
    pub fn find_clean_split(&self) -> Option<SubLocation> {
        self.pattern_splits.iter().find_map(|(pid, s)| {
            s.inner_offset.is_none().then_some(SubLocation {
                pattern_id: *pid,
                sub_index: s.sub_index,
            })
        })
    }
    //pub fn add_location_split(&mut self, location: SubLocation, split: Split) {
    //    self.pattern_splits.insert(location, split);
    //}
    //pub fn join_splits(&mut self, indexer: &mut Indexer, key: &PosKey) -> Split {
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
