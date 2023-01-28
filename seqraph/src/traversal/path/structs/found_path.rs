use crate::*;

/// used to represent results after traversal with any path
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FoundPath<R: ResultKind> {
    Complete(Child),
    Path(FoldResult<R>),
}
impl<
    'a: 'g,
    'g,
    R: ResultKind,
> FoundPath<R> {
    //fn num_path_segments(&self) -> usize {
    //    match self {
    //        Self::Complete(_) => 0,
    //        Self::Path(p) => HasMatchPaths::num_path_segments(p),
    //        Self::Prefix(p) => p.num_path_segments(),
    //        Self::Postfix(p) => p.num_path_segments(),
    //    }
    //}
    //#[allow(unused)]
    //#[track_caller]
    //pub fn unwrap_range(self) -> SearchPath {
    //    match self {
    //        Self::Path(path) => path,
    //        _ => panic!("Unable to unwrap {:?} as range.", self),
    //    }
    //}
    //#[allow(unused)]
    //#[track_caller]
    //pub fn get_range(&self) -> Option<&SearchPath> {
    //    match self {
    //        Self::Path(path) => Some(path),
    //        _ => None,
    //    }
    //}
    //#[allow(unused)]
    //fn is_complete(&self) -> bool {
    //    matches!(self, FoundPath::Complete(_))
    //}
}