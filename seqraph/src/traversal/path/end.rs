use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndPath {
    pub(crate) entry: ChildLocation,
    pub(crate) path: ChildPath,
    pub(crate) width: usize,
}
#[async_trait]
impl PathReduce for EndPath {
    async fn into_reduced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph().await;
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.path.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.path.push(location);
                break;
            }
        }
        self
    }
}
#[async_trait]
pub(crate) trait Retract: GraphEnd + EndPathMut + ExitMut + Send + Sync {
    async fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().await.expect_pattern_at(&location);
        D::pattern_index_prev(pattern, location.sub_index)
    }
    async fn retract<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
        R: ResultKind,
    >(&mut self, trav: &'a Trav) {
        let graph = trav.graph().await;
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at start of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end_path_mut().push(location);
                break;
            }
        }
        if self.end_path_mut().is_empty() {
            *self.exit_mut() = self.prev_exit_pos::<_, D, _>(trav).await.unwrap();
        }

    }
}
impl<T: GraphEnd + EndPathMut + ExitMut + Send + Sync> Retract for T {
}
impl GraphEntry for EndPath {
    fn entry(&self) -> ChildLocation {
        self.entry
    }
}
//impl BorderPath for EndPath {
//    fn path(&self) -> &[ChildLocation] {
//        self.path.borrow()
//    }
//    fn entry(&self) -> ChildLocation {
//        self.get_exit_location()
//    }
//}
impl<D: MatchDirection> PathBorder<D> for EndPath {
    type BorderDirection = Front;
}