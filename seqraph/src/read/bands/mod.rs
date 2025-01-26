use tracing::instrument;

use hypercontext_api::{
    graph::vertex::child::Child,
    path::{
        mutators::move_path::Advance,
        structs::query_range_path::PatternPrefixPath,
    },
};

use super::reader::context::ReadContext;
pub mod band;
pub mod overlaps;
//pub struct BandsContext {
//    pub graph: HypergraphRef,
//}
impl ReadContext {
    #[instrument(skip(self, sequence))]
    pub fn read(
        &mut self,
        mut sequence: PatternPrefixPath,
    ) {
        //println!("reading known bands");
        while let Some(next) = self.next_known_index(&mut sequence) {
            //println!("found next {:?}", next);
            let next = self.read_overlaps(next, &mut sequence).unwrap_or(next);
            self.append_index(next);
        }
    }
    #[instrument(skip(self, context))]
    fn next_known_index(
        &mut self,
        context: &mut PatternPrefixPath,
    ) -> Option<Child> {
        match self.insert_context().insert(context.clone()) {
            Ok((index, advanced)) => {
                *context = PatternPrefixPath::from(advanced);
                Some(index)
            }
            Err(_) => {
                context.advance(self);
                None
            }
        }
    }
}
