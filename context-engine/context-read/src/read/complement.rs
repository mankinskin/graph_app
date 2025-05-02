use derive_new::new;
use context_trace::graph::vertex::child::Child;

use super::{context::ReadContext, overlap::generator::ExpansionLink};

#[derive(Debug, new)]
pub struct ComplementBuilder {
    root: Child,
    link: ExpansionLink,
}

impl ComplementBuilder {
    pub fn build_context_index(self, trav: &ReadContext) -> Child {
        //
        //
        self.root.
    }

    //pub fn back_context_for_link(
    //    &mut self,
    //    link: &ExpansionLink,
    //) -> Pattern {
    //    //println!("building back context from path");
    //    //let (inner_back_ctx, _loc) = ctx
    //    //    .contexter::<SplitBack>()
    //    //    .try_context_path(
    //    //        //link.postfix_path.clone().into_context_path(),
    //    //        // FIXME: maybe mising root!!!
    //    //        link.postfix_path.clone().sub_path,
    //    //        //link.overlap,
    //    //    )
    //    //    .unwrap();

    //    let back_ctx = if let Some(last) = self.chain.bands.last() {
    //        let postfix_path = last.start_bound;
    //        //self.trav
    //        //    .read_pattern(last.back_context.borrow())
    //        //    .ok()
    //        //    //Some(self.graph.read_pattern(last.band.back_context.borrow()))
    //        //    .map(move |(back_ctx, _)| {
    //        //        last.back_context = vec![back_ctx];
    //        //        last.back_context.borrow()
    //        //    })
    //        Some(())
    //    } else {
    //        None
    //    }
    //    .unwrap_or_default();

    //    //DefaultDirection::context_then_inner(back_ctx, Child::new(0, 0)) //inner_back_ctx)
    //}
}
