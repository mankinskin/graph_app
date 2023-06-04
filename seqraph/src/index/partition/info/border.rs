use crate::*;

pub trait BoolPerfect: Default + Debug + Clone {
    type Result: BorderPerfect<Boolean = Self>;
    fn then_some(self, pid: PatternId) -> Self::Result;
}
impl BoolPerfect for bool {
    type Result = Option<PatternId>;
    fn then_some(self, pid: PatternId) -> Self::Result {
        self.then_some(pid)
    }
}
impl BoolPerfect for (bool, bool) {
    type Result = (Option<PatternId>, Option<PatternId>);
    fn then_some(self, pid: PatternId) -> Self::Result {
        (
            self.0.then_some(pid),
            self.1.then_some(pid),
        )
    }
}
pub trait BorderPerfect: Default + Debug + Clone {
    type Boolean: BoolPerfect<Result=Self>;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self;
    fn fold_or(&mut self, other: Self);
    fn complete(&self) -> Option<PatternId>;
}
impl BorderPerfect for Option<PatternId> {
    type Boolean = bool;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self {
        boolean.then_some(pid)
    }
    fn fold_or(&mut self, other: Self) {
        *self = self.or(other);
    }
    fn complete(&self) -> Option<PatternId> {
        *self
    }
}
impl BorderPerfect for (Option<PatternId>, Option<PatternId>) {
    type Boolean = (bool, bool);
    fn new((a, b): Self::Boolean, pid: PatternId) -> Self {
        (
            a.then_some(pid),
            b.then_some(pid),
        )
    }
    fn fold_or(&mut self, other: Self) {
        self.0.fold_or(other.0);
        self.1.fold_or(other.1);
    }
    fn complete(&self) -> Option<PatternId> {
        self.0.complete().and_then(|pid|
            self.1.complete().filter(|i| *i == pid)
        )
    }
}
pub type OffsetsOf<K> = <K as RangeRole>::Offsets;
pub type BooleanPerfectOf<K> = <<K as RangeRole>::Perfect as BorderPerfect>::Boolean;
pub type ChildrenOf<K> = <K as RangeRole>::Children;
pub type InnerRangeOf<K> = <K as RangeRole>::Range;
//pub type InnerOffsetsOf<K> = <<K as RangeRole>::Kind as RangeKind>::InnerOffsets;

pub struct BorderInfo<S>  {
    //child: Child,
    inner_index: usize,
    inner_offset: Option<NonZeroUsize>,
    _ty: std::marker::PhantomData<S>
}
impl BorderInfo<Left> {
    fn new(
        //cache: &SplitCache,
        pattern: &Pattern,
        pos: &PatternSplitPos,
    ) -> Self {
        if let Some(inner_offset) = pos.inner_offset {
            BorderInfo {
                inner_index: pos.sub_index + 1,
                //child: cache.expect_final_split(&SplitKey::new(pattern[pos.sub_index], inner_offset)).right,
                inner_offset: Some(inner_offset),
                _ty: Default::default(),
            }
        } else {
            BorderInfo {
                inner_index: pos.sub_index,
                //child: pattern[pos.sub_index],
                inner_offset: None,
                _ty: Default::default(),
            }
        }
    }
    fn outer_index(&self) -> usize {
        if self.inner_offset.is_none() {
            self.inner_index
        } else {
            self.inner_index - 1
        }
    }
}
impl BorderInfo<Right> {
    fn new(
        //cache: &SplitCache,
        pattern: &Pattern,
        pos: &PatternSplitPos,
    ) -> Self {
        if let Some(inner_offset) = pos.inner_offset {
            BorderInfo {
                inner_index: pos.sub_index,
                //child: cache.expect_final_split(&SplitKey::new(pattern[pos.sub_index], inner_offset)).left,
                inner_offset: Some(inner_offset),
                _ty: Default::default(),
            }
        } else {
            BorderInfo {
                inner_index: pos.sub_index,
                //child: pattern[pos.sub_index-1],
                inner_offset: None,
                _ty: Default::default(),
            }
        }
    }
    fn outer_index(&self) -> usize {
        self.inner_index
    }
}

//pub trait PartitionInnerInfo<K: RangeRole>: Sized {
//    //fn outer_children(&self) -> Option<ChildrenOf<K>>;
//    fn perfect(&self) -> BooleanPerfectOf<K>;
//    //fn inner_offsets(&self) -> InnerOffsetsOf<K>;
//    fn inner_range(&self) -> InnerRangeOf<K>;
//    fn has_inner_range(&self, pattern: &Pattern) -> bool;
//    fn inner_info(&self, pattern: &Pattern) -> Option<InnerRangeInfo<K>> {
//        self.has_inner_range(pattern).then_some(())
//            .and_then(|_| self.outer_children())
//            .map(|children|
//                InnerRangeInfo {
//                    range: self.inner_range(),
//                    offsets: self.inner_offsets(),
//                    children,
//                }
//            )
//    }
//
//    fn join_inner_info(
//        self,
//        pid: PatternId,
//        pattern: &Pattern,
//    ) -> Result<PatternRangeInfo<K>, Child> {
//        let perfect = self.perfect();
//        //let outer_range = left_border.outer_index()..right_border.outer_index();
//        if let Some(info) = self.inner_info(pattern) {
//            let inner = pattern.get(info.range).unwrap();
//            let delta = inner.len().saturating_sub(1);
//            Ok(PatternRangeInfo {
//                pattern_id: pid,
//                info: RangeInfo {
//                    inner_range: Some(info),
//                    delta,
//                },
//                perfect,
//            })
//        } else {
//
//        }
//    }
//}
//impl PartitionInnerInfo<Pre> for BorderInfo<Right> {
//    //fn outer_children(&self) -> Option<ChildrenOf<Pre>> {
//    //    self.inner_offset.is_some().then_some(self.child)
//    //}
//    fn perfect(&self) -> BooleanPerfectOf<Pre> {
//        self.inner_offset.is_none()
//    }
//    //fn inner_offsets(&self) -> InnerOffsetsOf<Pre> {
//    //    self.inner_offset
//    //}
//    fn inner_range(&self) -> InnerRangeOf<Pre> {
//        0..self.inner_index
//    }
//    fn has_inner_range(&self, _pattern: &Pattern) -> bool {
//        self.inner_index > 1
//    }
//}
//impl PartitionInnerInfo<Post> for BorderInfo<Left> {
//    //fn outer_children(&self) -> Option<ChildrenOf<Post>> {
//    //    self.inner_offset.is_some().then_some(self.child)
//    //}
//    fn perfect(&self) -> BooleanPerfectOf<Post> {
//        self.inner_offset.is_none()
//    }
//    //fn inner_offsets(&self) -> InnerOffsetsOf<Post> {
//    //    self.inner_offset
//    //}
//    fn inner_range(&self) -> InnerRangeOf<Post> {
//        self.inner_index..
//    }
//    fn has_inner_range(&self, pattern: &Pattern) -> bool {
//        pattern.len() - self.inner_index > 1
//    }
//}
//impl PartitionInnerInfo<In> for (BorderInfo<Left>, BorderInfo<Right>) {
//    //fn outer_children(&self) -> Option<ChildrenOf<Infix>> {
//    //    match (self.0.perfect(), self.1.perfect()) {
//    //        (true, true) => None,
//    //        (false, false) => Some(
//    //            InfixChildren::Both(self.0.child, self.1.child),
//    //        ),
//    //        (true, false) => Some(
//    //            InfixChildren::Right(self.1.child),
//    //        ),
//    //        (false, true) => Some(
//    //            InfixChildren::Left(self.0.child),
//    //        ),
//    //    }
//    //}
//    fn perfect(&self) -> BooleanPerfectOf<In> {
//        (
//            self.0.perfect(),
//            self.1.perfect(),
//        )
//    }
//    //fn inner_offsets(&self) -> InnerOffsetsOf<Infix> {
//    //    (
//    //        self.0.inner_offset,
//    //        self.1.inner_offset,
//    //    )
//    //}
//    fn inner_range(&self) -> InnerRangeOf<In> {
//        self.0.inner_index..self.1.inner_index
//    }
//    fn has_inner_range(&self, _pattern: &Pattern) -> bool {
//        self.1.inner_index - self.0.inner_index > 1
//    }
//}

//pub trait PartitionBorders<K: RangeRole>: PartitionInnerInfo<K> {
//    type Splits;
//    fn border_info(
//        //cache: &SplitCache,
//        pattern: &Pattern,
//        splits: &Self::Splits,
//    ) -> Self;
//}
//impl<K: RangeRole<Borders=Self>> PartitionBorders<K> for BorderInfo<Left> 
//    where BorderInfo<Left>: PartitionInnerInfo<K>,
//{
//    type Splits = PatternSplitPos;
//    fn border_info(
//        //cache: &SplitCache,
//        pattern: &Pattern,
//        splits: &Self::Splits,
//    ) -> Self {
//        Self::new(pattern, splits)
//    }
//}
//impl<K: RangeRole<Borders=Self>> PartitionBorders<K> for BorderInfo<Right>
//    where BorderInfo<Right>: PartitionInnerInfo<K>
//{
//    type Splits = PatternSplitPos;
//    fn border_info(
//        //cache: &SplitCache,
//        pattern: &Pattern,
//        splits: &Self::Splits,
//    ) -> Self {
//        Self::new(pattern, splits)
//    }
//}
//impl PartitionBorders<In> for (BorderInfo<Left>, BorderInfo<Right>)
//{
//    type Splits = (
//        <BorderInfo::<Left> as PartitionBorders<Post>>::Splits,
//        <BorderInfo::<Right> as PartitionBorders<Pre>>::Splits,
//    );
//    fn border_info(
//        //cache: &SplitCache,
//        pattern: &Pattern,
//        splits: &Self::Splits,
//    ) -> Self {
//        (
//            BorderInfo::<Left>::new(pattern, &splits.0),
//            BorderInfo::<Right>::new(pattern, &splits.1),
//        )
//    }
//}