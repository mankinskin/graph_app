use super::*;

pub(crate) trait ReduciblePath: Clone + EndPathMut + ExitMut + End {
    fn prev_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize>;
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.end_path_mut().pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end_path_mut().push(location);
                break;
            }
        }
        if self.end_path_mut().is_empty() {
            *self.exit_mut() = self.prev_exit_pos::<_, D, _>(trav).unwrap();
        }
        self
    }
}