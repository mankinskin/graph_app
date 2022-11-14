use futures::future::OptionFuture;

use crate::*;
use super::*;

//pub(crate) trait Indexing<'a: 'g, 'g, T: Tokenize, D: IndexDirection>: TraversableMut<'a, 'g, T> {
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Indexer<T, D> {
    pub async fn index_found(
        &'a mut self,
        found: FoundPath,
    ) -> Child {
        //println!("indexing found path {:#?}", found);
        match found {
            FoundPath::Range(path) => self.index_range_path(path).await,
            FoundPath::Prefix(path) => self.index_prefix_path(path).await,
            FoundPath::Postfix(path) => self.index_postfix_path(path).await,
            FoundPath::Complete(c) => c
        }
    }
    async fn index_prefix_path(
        &'a mut self,
        path: EndPath,
    ) -> Child {
        self.splitter::<IndexFront>().single_path_split(
            std::iter::once(&path.get_exit_location()).chain(
                path.end_path().into_iter()
            )
        )
        .await
        .map(|split| split.inner)
        .expect("EndPath for complete path!")
    }
    async fn index_postfix_path(
        &'a mut self,
        path: StartPath,
    ) -> Child {
        self.splitter::<IndexBack>().single_path_split(
            path.start_path().into_iter().chain(
                std::iter::once(&path.entry()),
            )
        )
        .await
        .map(|split| split.inner)
        .expect("StartPath for complete path!")
    }
    #[instrument(skip(self, path))]
    async fn index_range_path(
        &'a mut self,
        path: SearchPath,
    ) -> Child {
        let entry = path.start.entry();
        let entry_pos = path.start.get_entry_pos();
        let exit_pos = path.end.get_exit_pos();

        let location = entry.into_pattern_location();

        let range = D::wrapper_range(entry_pos, exit_pos);
        self.graph().await.validate_pattern_indexing_range_at(&location, entry_pos, exit_pos).unwrap();
        let inserted = self.graph_mut().await.insert_range_in(
                location,
                range,
            );
        let (wrapper, pattern, location) = if let Ok(wrapper) = inserted {
            let (pid, pattern) = self.graph().await.expect_any_child_pattern(wrapper)
                .pipe(|(&pid, pattern)|
                    (pid, pattern.clone())
                );
            let location = wrapper.to_pattern_location(pid);
            (wrapper, pattern, location)
        } else {
            let wrapper = location.parent;
            let pattern = self.graph().await.expect_child_pattern(wrapper, location.pattern_id).clone();
            (wrapper, pattern, location)
        };

        let head_pos = D::head_index(pattern.borrow());
        let last_pos = D::last_index(pattern.borrow());

        let mut head_contexter = self.contexter::<IndexBack>();
        let head_split = OptionFuture::from(self.splitter::<IndexBack>().single_path_split(
            path.start.start_path().to_vec()
        ).await
        .map(async move |split| (
            split.inner,
            head_contexter.try_context_path(
                split.path.into_iter().chain(
                    std::iter::once(split.location)
                ),
                //split.inner,
            ).await.unwrap().0
        ))).await;

        let mut last_contexter = self.contexter::<IndexFront>();
        let last_split = OptionFuture::from(
            self.splitter::<IndexFront>().single_path_split(
                path.end.end_path().to_vec()
            )
            .await
            .map(async move |split| (
                split.inner,
                last_contexter.try_context_path(
                    std::iter::once(split.location).chain(
                        split.path.into_iter()
                    ),
                    //split.inner,
                ).await.unwrap().0
            ))).await;

        let mut graph = self.graph_mut().await;
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