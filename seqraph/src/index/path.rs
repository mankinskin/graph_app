
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Pather<T, D, Side> {
    /// replaces context in pattern at location with child and returns it with new location
    pub(crate) fn splitter(&self) -> Splitter<T, D, Side> {
        Splitter::new(self.indexer.clone())
    }
    fn index_half<
        S: RelativeSide<D, Side>,
    >(
        &'a mut self,
        location: impl Borrow<ChildLocation>,
    ) -> Option<(Child, ChildLocation)> {
        self.splitter().entry_perfect_split::<S, _>(location)
                    .map(|split|
                        (split.inner, split.location)
                    )
    }
    pub fn try_index_path<
        S: RelativeSide<D, Side>,
    >(
        &'a mut self,
        path: impl ContextPath,
    ) -> Option<(Child, ChildLocation)> {
        Side::bottom_up_path_iter(path).fold(None, |prev, location| {
            let location = location.borrow();
            if let Some((prev, prev_loc)) = prev {
                let (prev_opposite, _) = self.index_half::<<S as RelativeSide<_, _>>::Opposite>(prev_loc);
                let (local, local_location) = self.index_half::<S>(location);
                // join prev and local
                let (back, front) = Side::context_inner_order(&local, &prev);
                let context = self.indexer.index_pattern([back[0], front[0]]).unwrap().0;
                let pid = self.graph_mut().add_pattern_with_update(location, Side::concat_inner_and_context(inner, context));

                let (sub_index, _) = Side::back_front_order(0, 1);
                Some((context, ChildLocation {
                    parent: inner_location.parent,
                    pattern_id: pid,
                    sub_index,
                }))
            } else {
                self.index_half::<S>(location)
            }
        })
    }
}