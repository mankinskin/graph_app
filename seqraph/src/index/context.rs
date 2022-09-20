use super::*;

pub(crate) trait IndexContext<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexing<'a, 'g, T, D> {
    /// * `location` - Points to child to index the context of
    fn context_path_segment(
        &'a mut self,
        location: ChildLocation
    ) -> (Child, ChildLocation) {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&location);
        let context = Side::split_context(&pattern, location.sub_index);
        if context.len() < 2 {
            if context.is_empty() {
                assert!(!context.is_empty());
            }
            (*context.into_iter().next().unwrap(), location)
        } else {
            let c = graph.index_pattern(context);
            let range = Side::context_range(location.sub_index);
            graph.replace_in_pattern(location, range, c);
            (c, location.to_child_location(Side::inner_pos_after_context_indexed(location.sub_index)))
        }
    }
    /// * `entry` - Points to child to index the context of
    /// * `context_path` - List of child locations pointing to entry to build the nested context structure
    fn context_path(
        &'a mut self,
        entry: ChildLocation,
        mut context_path: Vec<ChildLocation>,
        inner: Child,
    ) -> (Child, ChildLocation) {
        let mut graph = self.graph_mut();
        let mut acc: Option<Child> = None;
        while let Some(location) = context_path.pop() {
            let (context, _inner_location) = IndexContext::<_, _, Side>::context_path_segment(&mut *graph, location);
            if let Some(acc) = &mut acc {
                let (back, front) = Side::context_inner_order(&context, &acc);
                let context = graph.index_pattern([back[0], front[0]]);
                graph.add_pattern_with_update(location, Side::concat_inner_and_context(inner, context));
                *acc = context;
            } else {
                acc = Some(context);
            }
        }
        let (context, inner_location)
            = IndexContext::<_, _, Side>::context_path_segment(&mut *graph, entry);
        if let Some(acc) = acc {
            let (back, front) = Side::context_inner_order(&context, &acc);
            let context = graph.index_pattern([back[0], front[0]]);
            let pid = graph.add_pattern_with_update(entry, Side::concat_inner_and_context(inner, context));
            let (sub_index, _) = Side::back_front_order(0, 1);
            (context, ChildLocation {
                parent: inner_location.parent,
                pattern_id: pid,
                sub_index,
            })
        } else {
            (context, inner_location)
        }
    }
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Trav: Indexing<'a, 'g, T, D>,
    S: IndexSide<D>,
> IndexContext<'a, 'g, T, D, S> for Trav {}