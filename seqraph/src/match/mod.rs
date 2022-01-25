use crate::{
    search::*,
    vertex::*,
    *,
};
mod matcher;
pub use matcher::*;
mod match_direction;
pub use match_direction::*;
//mod async_matcher;
//pub use async_matcher::*;
//mod async_match_direction;
//pub use async_match_direction::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NoMatch {
    EmptyPatterns,
    NoParents,
    Mismatch(MismatchPath),
    NoChildPatterns,
    NotFound(Pattern),
    NoMatchingParent(VertexIndex),
    SingleIndex,
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
}

//#[derive(Clone, Debug, PartialEq, Eq)]
//pub struct MatchPath {
//    pub(crate) path: ChildPath,
//    pub(crate) remainder: MatchRemainder,
//}
//impl MatchPath {
//    fn flip_remainder(self) -> Self {
//        Self {
//            path: self.path,
//            remainder: self.remainder.flip(),
//        }
//    }
//    fn complete() -> Self {
//        Self {
//            path: vec![],
//            remainder: MatchRemainder::None,
//        }
//    }
//}
//#[derive(Clone, Debug, PartialEq, Eq)]
//pub enum MatchRemainder {
//    Left(Pattern),
//    Right(Pattern),
//    None,
//}
//impl MatchRemainder {
//    fn flip(self) -> Self {
//        match self {
//            Self::Right(p) => Self::Left(p),
//            Self::Left(p) => Self::Right(p),
//            Self::None => Self::None,
//        }
//    }
//}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrownPath {
    pub(crate) path: ChildPath,
    pub(crate) remainder: GrowthRemainder,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrowthRemainder {
    Query(Pattern),
    Child(Pattern),
    None,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MismatchPath {
    pub path: ChildPath,
    pub child: Pattern,
    pub query: Pattern,
}
//impl MismatchPath {
//    fn flip_remainder(self) -> Self {
//        Self {
//            path: self.path,
//            left: self.right,
//            right: self.left,
//        }
//    }
//}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildLocation {
    pub(crate) parent: Child,
    pub(crate) pattern_id: PatternId,
    pub(crate) sub_index: usize,
}
impl ChildLocation {
    pub fn new(parent: impl AsChild, pattern_id: PatternId, sub_index: usize) -> Self {
        Self {
            parent: parent.as_child(),
            pattern_id,
            sub_index,
        }
    }
}
pub type ChildPath = Vec<ChildLocation>;
//pub type MatchResult = Result<MatchPath, MismatchPath>;


impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn right_matcher(&'a self) -> Matcher<'a, T, Right> {
        Matcher::new(self)
    }
    pub fn left_matcher(&'a self) -> Matcher<'a, T, Left> {
        Matcher::new(self)
    }
    //pub fn compare_pattern_postfix<C: AsChild>(
    //    &self,
    //    a: impl IntoPattern<Item = C>,
    //    b: impl IntoPattern<Item = C>,
    //) -> MatchResult {
    //    self.left_matcher().compare(a, b)
    //}
    //pub fn compare_pattern_prefix(
    //    &self,
    //    a: impl IntoPattern<Item = impl AsChild>,
    //    b: impl IntoPattern<Item = impl AsChild>,
    //) -> MatchResult {
    //    self.right_matcher().compare(a, b)
    //}
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::tests::context,
        Child,
    };
    use pretty_assertions::{
        assert_eq,
    };
    //#[test]
    //fn compare_pattern_prefix() {
    //    let Context {
    //        graph,
    //        a,
    //        b,
    //        c,
    //        d,
    //        e,
    //        f,
    //        g,
    //        h,
    //        i,
    //        ab,
    //        bc,
    //        bc_id,
    //        cd,
    //        cd_id,
    //        bcd,
    //        b_cd_id,
    //        abc,
    //        abcd,
    //        a_bcd_id,
    //        abc_d_id,
    //        ef,
    //        efghi,
    //        ..
    //    } = &*context();
    //    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    //    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    //    let abc_d_pattern = vec![Child::new(abc, 3), Child::new(d, 1)];
    //    let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    //    let ab_c_d_pattern = vec![Child::new(ab, 2), Child::new(c, 1), Child::new(d, 1)];
    //    let abcd_pattern = vec![Child::new(abcd, 4)];
    //    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    //    let bc_pattern = vec![Child::new(bc, 2)];
    //    let a_d_c_pattern = vec![Child::new(a, 1), Child::new(d, 1), Child::new(c, 1)];
    //    let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
    //    assert_eq!(
    //        graph.compare_pattern_prefix(vec![e, f, g, h, i], vec![efghi]),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(vec![ef], vec![e, f]),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(vec![e, f], vec![ef]),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(vec![efghi], vec![e, f, g, h, i]),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&bc_pattern, vec![*b]),
    //        Ok(MatchPath {
    //            path: vec![
    //                ChildLocation::new(bc, *bc_id, 0),
    //            ],
    //            remainder: MatchRemainder::Left(vec![*c]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_bc_pattern, &ab_c_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&ab_c_pattern, &a_bc_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_b_c_pattern, &a_bc_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_b_c_pattern, &a_b_c_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&ab_c_pattern, &a_b_c_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_bc_d_pattern, &ab_c_d_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&abc_d_pattern, &a_bc_d_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_bc_d_pattern, &abc_d_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_bc_d_pattern, &abcd_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&abcd_pattern, &a_bc_d_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_b_c_pattern, &abcd_pattern),
    //        Ok(MatchPath {
    //            path: vec![
    //                ChildLocation::new(abcd, *a_bcd_id, 1),
    //                ChildLocation::new(bcd, *b_cd_id, 1),
    //                ChildLocation::new(cd, *cd_id, 0),
    //            ],
    //            remainder: MatchRemainder::Right(vec![*d]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&ab_c_d_pattern, &a_bc_pattern),
    //        Ok(MatchPath {
    //            path: vec![],
    //            remainder: MatchRemainder::Left(vec![*d]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&a_bc_pattern, &ab_c_d_pattern),
    //        Ok(MatchPath {
    //            path: vec![],
    //            remainder: MatchRemainder::Right(vec![*d]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&bc_pattern, &abcd_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: bc_pattern.clone(),
    //            right: abcd_pattern.clone(),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&b_c_pattern, &a_bc_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: b_c_pattern.clone(),
    //            right: a_bc_pattern.clone(),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_prefix(&b_c_pattern, &a_d_c_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: b_c_pattern.clone(),
    //            right: a_d_c_pattern.clone(),
    //        })
    //    );
    //}
    //#[test]
    //fn compare_pattern_postfix() {
    //    let Context {
    //        graph,
    //        a,
    //        b,
    //        c,
    //        d,
    //        ab,
    //        bc,
    //        abc,
    //        a_bc_id,
    //        abcd,
    //        abc_d_id,
    //        ..
    //     } = &*context();
    //    let a_bc_pattern = vec![*a, *bc];
    //    let ab_c_pattern = vec![*ab, *c];
    //    let abc_d_pattern = vec![*abc, *d];
    //    let a_bc_d_pattern = vec![*a, *bc, *d];
    //    let ab_c_d_pattern = vec![*ab, *c, *d];
    //    let abcd_pattern = vec![*abcd];
    //    let b_c_pattern = vec![*b, *c];
    //    let b_pattern = vec![*b];
    //    let bc_pattern = vec![*bc];
    //    let a_d_c_pattern = vec![*a, *d, *c];
    //    let a_b_c_pattern = vec![*a, *b, *c];
    //    let a_b_pattern = vec![*a, *b];
    //    let bc_d_pattern = vec![*bc, *d];
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_bc_pattern, &ab_c_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&ab_c_pattern, &a_bc_pattern),
    //        Ok(MatchPath::complete())
    //    );

    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_b_pattern, &b_pattern),
    //        Ok(MatchPath {
    //            path: vec![],
    //            remainder: MatchRemainder::Left(vec![*a]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_b_c_pattern, &a_bc_pattern),
    //        Ok(MatchPath::complete())
    //    );

    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_b_c_pattern, &a_b_c_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&ab_c_pattern, &a_b_c_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_bc_d_pattern, &ab_c_d_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&abc_d_pattern, &a_bc_d_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&bc_pattern, &abcd_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: bc_pattern.clone(),
    //            right: abcd_pattern.clone(),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&b_c_pattern, &a_bc_pattern),
    //        Ok(MatchPath {
    //            path: vec![],
    //            remainder: MatchRemainder::Right(vec![*a]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&b_c_pattern, &a_d_c_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: vec![*b],
    //            right: vec![*a, *d],
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_bc_d_pattern, &abc_d_pattern),
    //        Ok(MatchPath::complete())
    //    );

    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_bc_d_pattern, &abcd_pattern),
    //        Ok(MatchPath::complete())
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&abcd_pattern, &a_bc_d_pattern),
    //        Ok(MatchPath::complete())
    //    );

    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_b_c_pattern, &abcd_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: a_b_c_pattern.clone(),
    //            right: abcd_pattern.clone(),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&ab_c_d_pattern, &a_bc_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: ab_c_d_pattern.clone(),
    //            right: a_bc_pattern.clone(),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&a_bc_pattern, &ab_c_d_pattern),
    //        Err(MismatchPath {
    //            path: vec![],
    //            left: a_bc_pattern.clone(),
    //            right: ab_c_d_pattern.clone(),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&bc_d_pattern, &ab_c_d_pattern),
    //        Ok(MatchPath {
    //            path: vec![],
    //            remainder: MatchRemainder::Right(vec![*a]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&bc_d_pattern, &abc_d_pattern),
    //        Ok(MatchPath {
    //            path: vec![
    //                ChildLocation::new(abc, *a_bc_id, 1),
    //            ],
    //            remainder: MatchRemainder::Right(vec![*a]),
    //        })
    //    );
    //    assert_eq!(
    //        graph.compare_pattern_postfix(&abcd_pattern, &bc_d_pattern),
    //        Ok(MatchPath {
    //            path: vec![
    //                ChildLocation::new(abc, *a_bc_id, 1),
    //                ChildLocation::new(abcd, *abc_d_id, 1),
    //            ],
    //            remainder: MatchRemainder::Left(vec![*a]),
    //        })
    //    );
    //}
}
