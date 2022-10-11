use crate::*;
use super::*;

pub(crate) trait Indexing<'a: 'g, 'g, T: Tokenize, D: IndexDirection>: TraversableMut<'a, 'g, T> {
    fn index_found(
        &'a mut self,
        found: FoundPath,
    ) -> Child {
        match found {
            FoundPath::Range(path) => self.index_range_path(path),
            FoundPath::Prefix(path) => self.index_prefix_path(path),
            FoundPath::Postfix(path) => self.index_postfix_path(path),
            FoundPath::Complete(c) => c
        }
    }
    fn index_prefix_path(
        &'a mut self,
        path: EndPath,
    ) -> Child {
        IndexSplit::<_, D, IndexFront>::single_entry_split(
            self,
            path.get_exit_location(),
            path.end_path().to_vec()
        )
        .map(|split| split.inner)
        .expect("EndPath for complete path!")
    }
    fn index_postfix_path(
        &'a mut self,
        path: StartPath,
    ) -> Child {
        IndexSplit::<_, D, IndexBack>::single_entry_split(
            self,
            path.get_entry_location(),
            path.start_path().to_vec()
        )
        .map(|split| split.inner)
        .expect("StartPath for complete path!")
    }
    fn index_range_path(
        &'a mut self,
        path: SearchPath,
    ) -> Child {
        let entry = path.start.get_entry_location();
        let entry_pos = path.start.get_entry_pos();
        let exit_pos = path.end.get_exit_pos();
        let mut graph = self.graph_mut();

        //// a little bit dirty, path should have typing for this
        //if entry_pos == exit_pos && path.start.path().is_empty() && path.end.path().is_empty() {
        //    return graph.expect_child_at(&location);
        //}
        let location = entry.into_pattern_location();

        let range = D::wrapper_range(entry_pos, exit_pos);
        graph.validate_pattern_indexing_range_at(&location, entry_pos, exit_pos).unwrap();
        let (wrapper, pattern, location) = if let Ok(wrapper) =
            graph.index_range_in(
                location,
                range,
            ) {
                let (pid, pattern) = wrapper.expect_child_patterns(&*graph).into_iter().next().unwrap();
                let location = wrapper.to_pattern_location(pid);
                (wrapper, pattern, location)
            } else {
                let wrapper = location.parent;
                let pattern = wrapper.expect_child_pattern(&*graph, location.pattern_id);
                (wrapper, pattern, location)
            };

        let head_pos = D::head_index(pattern.borrow());
        let last_pos = D::last_index(pattern.borrow());

        let head_split = IndexSplit::<_, D, IndexBack>::single_path_split(
            &mut *graph,
            path.start.start_path().to_vec()
        ).map(|split| (
            split.inner,
            IndexContext::<_, D, IndexBack>::context_entry_path(
                &mut *graph,
                split.location,
                split.path,
                split.inner,
            ).0
        ));
        let last_split =
            IndexSplit::<_, D, IndexFront>::single_path_split(
                &mut *graph,
                path.end.end_path().to_vec()
            ).map(|split| (
                split.inner,
                IndexContext::<_, D, IndexFront>::context_entry_path(
                    &mut *graph,
                    split.location,
                    split.path,
                    split.inner,
                ).0
            ));

        let res = match (head_split, last_split) {
            (Some((head_inner, head_context)), Some((last_inner, last_context))) => {
                let range = D::inner_context_range(head_pos, last_pos);
                let inner = graph.index_range_in(
                    location,
                    range,
                ).ok();
                let target = graph.insert_pattern(
                    D::concat_context_inner_context(
                        head_inner,
                        inner.as_ref().map(std::slice::from_ref).unwrap_or_default(),
                        last_inner
                    )
                ).unwrap();
                graph.add_pattern_with_update(
                    wrapper,
                    D::concat_context_inner_context(head_context, target, last_context)
                );
                target
            },
            (Some((head_inner, head_context)), None) => {
                let range = 
                    <IndexBack as IndexSide<D>>::inner_context_range(head_pos);
                let inner_context = graph.index_range_in_or_default(
                    location,
                    range,
                ).unwrap();
                // |context, [inner, inner_context]|
                let target = graph.insert_pattern(
                    D::inner_then_context(head_inner, inner_context)
                ).unwrap();
                // |context, target|
                graph.add_pattern_with_update(
                    wrapper,
                    D::context_then_inner(head_context, target)
                );
                target
            },
            (None, Some((last_inner, last_context))) => {
                let range = 
                    <IndexFront as IndexSide<D>>::inner_context_range(last_pos);
                let inner_context = graph.index_range_in_or_default(
                    location,
                    range,
                ).unwrap();
                // |[inner_context, inner], context|
                let target = graph.insert_pattern(
                    D::context_then_inner(inner_context, last_inner)
                ).unwrap();
                // |target, context|
                graph.add_pattern_with_update(
                    wrapper,
                    D::inner_then_context(target, last_context)
                );
                target
            },
            (None, None) => wrapper,
        };
        graph.validate_expansion(entry.parent);
        res
    }
}
impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Trav: TraversableMut<'a, 'g, T>,
> Indexing<'a, 'g, T, D> for Trav {}
