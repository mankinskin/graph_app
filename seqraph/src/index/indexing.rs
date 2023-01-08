use crate::*;

//pub trait Indexing<T: Tokenize, D: IndexDirection>: TraversableMut<T> {
impl<T: Tokenize, D: IndexDirection> Indexer<T, D> {
    pub fn index_found(
        &mut self,
        path: FoundPath,
    ) -> Child {
        //println!("indexing found path {:#?}", path);
        match path {
            FoundPath::Range(path) => self.index_range_path(path),
            FoundPath::Prefix(path) => self.index_prefix_path(path),
            FoundPath::Postfix(path) => self.at_postfix_path(path),
            FoundPath::Complete(c) => c
        }
    }
    fn index_prefix_path(
        &mut self,
        path: ChildPath<End>,
    ) -> Child {
        self.splitter::<IndexFront>().single_path_split(
            std::iter::once(&path.child_location()).chain(
                path.path().into_iter()
            ).collect_vec(),
        )
        
        .map(|split| split.inner)
        .expect("ChildPath for complete path!")
    }
    fn at_postfix_path(
        &mut self,
        path: ChildPath<Start>,
    ) -> Child {
        self.splitter::<IndexBack>().single_path_split(
            path.path().into_iter().chain(
                std::iter::once(&path.child_location())
            ).collect_vec(),
        )
        
        .map(|split| split.inner)
        .expect("ChildPath for complete path!")
    }
    #[instrument(skip(self, path))]
    fn index_range_path(
        &mut self,
        path: SearchPath,
    ) -> Child {
        let entry = path.start.child_location();
        let entry_pos = ChildPos::<Start>::child_pos(&path);
        let exit_pos = ChildPos::<End>::child_pos(&path);

        let location = entry.into_pattern_location();

        let range = D::wrapper_range(entry_pos, exit_pos);
        self.graph().validate_pattern_indexing_range_at(&location, entry_pos, exit_pos).unwrap();
        let inserted = self.graph_mut().insert_range_in(
                location,
                range,
            );
        let (wrapper, pattern, location) = if let Ok(wrapper) = inserted {
            let (pid, pattern) = self.graph().expect_any_child_pattern(wrapper)
                .pipe(|(&pid, pattern)|
                    (pid, pattern.clone())
                );
            let location = wrapper.to_pattern_location(pid);
            (wrapper, pattern, location)
        } else {
            let wrapper = location.parent;
            let pattern = self.graph().expect_child_pattern(wrapper, location.pattern_id).clone();
            (wrapper, pattern, location)
        };

        let head_pos = D::head_index(pattern.borrow());
        let last_pos = D::last_index(pattern.borrow());

        let mut head_contexter = self.contexter::<IndexBack>();
        let head_split = self.splitter::<IndexBack>().single_path_split(
            path.start.path().to_vec()
        )
        .map(|split| (
            split.inner,
            head_contexter.try_context_path(
                split.path.into_iter().chain(
                    std::iter::once(split.location)
                ).collect_vec(),
                //split.inner,
            ).unwrap().0
        ));

        let mut last_contexter = self.contexter::<IndexFront>();
        let last_split = 
            self.splitter::<IndexFront>().single_path_split(
                path.end.path().to_vec()
            )
            .map(|split| (
                split.inner,
                last_contexter.try_context_path(
                    std::iter::once(split.location).chain(
                        split.path.into_iter()
                    ).collect_vec(),
                    //split.inner,
                ).unwrap().0
            ));

        let mut graph = self.graph_mut();
        let res = match (head_split, last_split) {
            (Some((head_inner, head_context)), Some((last_inner, last_context))) => {
                let range = D::inner_context_range(head_pos, last_pos);
                let inner = graph.insert_range_in(
                    location,
                    range,
                ).ok();
                let target = graph.insert_pattern(
                    D::concat_context_inner_context(
                        head_inner,
                        inner.as_ref().map(std::slice::from_ref).unwrap_or_default(),
                        last_inner
                    )
                );
                graph.add_pattern_with_update(
                    wrapper,
                    D::concat_context_inner_context(head_context, target, last_context)
                );
                target
            },
            (Some((head_inner, head_context)), None) => {
                let range = 
                    <IndexBack as IndexSide<D>>::inner_context_range(head_pos);
                let inner_context = graph.insert_range_in_or_default(
                    location,
                    range,
                ).unwrap();
                // |context, [inner, inner_context]|
                let target = graph.insert_pattern(
                    D::inner_then_context(head_inner, inner_context)
                );
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
                let inner_context = graph.insert_range_in_or_default(
                    location,
                    range,
                ).unwrap();
                // |[inner_context, inner], context|
                let target = graph.insert_pattern(
                    D::inner_then_context(inner_context, last_inner)
                );
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