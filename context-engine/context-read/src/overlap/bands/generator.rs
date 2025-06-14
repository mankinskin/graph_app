use crate::{
    context::ReadContext,
    overlap::bands::LinkedBands,
};
use context_insert::insert::{
    result::IndexWithPath,
    ToInsertContext,
};
use context_trace::{
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
        wide::Wide,
    },
    path::{
        accessors::role::{
            End,
            Start,
        },
        mutators::append::PathAppend,
        structs::{
            role_path::RolePath,
            rooted::role_path::PatternEndPath,
            sub_path::SubPath,
        },
    },
    trace::child::bands::HasChildRoleIters,
};
use derive_more::{
    Deref,
    DerefMut,
    From,
};

use super::band::Band;

pub trait ChainAppendage: Sized {
    fn append_to_chain(
        self,
        chain: &mut LinkedBands,
    ) {
        chain.bands.insert(self.into_band());
    }
    fn into_band(self) -> Band;
}
impl ChainAppendage for Band {
    fn into_band(self) -> Band {
        self
    }
}
impl ChainAppendage for (usize, Band) {
    fn into_band(self) -> Band {
        self.1
    }
}
#[derive(Debug, From)]
pub enum ChainOp {
    Expansion(usize, IndexWithPath),
    Cap(BandCap),
}
#[derive(Debug)]
pub struct BandCap {
    pub postfix_path: RolePath<End>,
    pub expansion: Child,
    pub start_bound: usize,
}
#[derive(Debug)]
pub struct ExpansionLink {
    pub prefix_path: RolePath<Start>,
    pub expansion: Child,
    pub start_bound: usize,
}

#[derive(Debug, Deref, DerefMut)]
pub struct ChainGenerator<'a> {
    pub trav: ReadContext,
    pub cursor: &'a mut PatternEndPath,
    #[deref]
    #[deref_mut]
    pub chain: LinkedBands,
}

impl<'a> Iterator for ChainGenerator<'a> {
    type Item = ChainOp;

    fn next(&mut self) -> Option<Self::Item> {
        self.expand_step()
    }
}
impl<'a> ChainGenerator<'a> {
    pub fn new(
        trav: ReadContext,
        cursor: &'a mut PatternEndPath,
        chain: LinkedBands,
    ) -> Self {
        Self {
            trav,
            cursor,
            chain,
        }
    }

    /// expand postfix into cursor context
    fn postfix_step(
        &self,
        start_pos: usize,
        postfix_path: &mut RolePath<End>,
        postfix_location: ChildLocation,
        postfix: Child,
    ) -> Option<ChainOp> {
        postfix_path.path_append(postfix_location);

        let cursor: PatternEndPath = self.cursor.clone();
        let primer = postfix_path.clone();

        match ToInsertContext::<IndexWithPath>::insert_or_get_complete(
            &self.trav.graph,
            cursor,
        ) {
            Ok(expansion) => Some(ChainOp::Expansion(start_pos, expansion)),
            Err(_) => match self.chain.ends_at(start_pos) {
                Some(_) => Some(ChainOp::Cap(BandCap {
                    postfix_path: postfix_path.clone(),
                    expansion: postfix,
                    start_bound: start_pos,
                })),
                _ => None,
            },
        }
    }
    /// find largest expandable postfix
    fn expand_step(&mut self) -> Option<ChainOp> {
        let last = self.chain.last();
        let last_end = last.band.postfix();

        // TODO: Replace with Child Trace Iter
        let mut postfix_iter = last_end.postfix_iter(self.trav.clone());
        let mut postfix_path = {
            let postfix_location = postfix_iter.next().unwrap().0;
            RolePath::from(SubPath::new(postfix_location.sub_index))
        };
        postfix_iter.find_map(|(postfix_location, postfix)| {
            let last_end_bound = last.band.end_bound;
            let start_bound = last_end_bound - postfix.width();
            self.postfix_step(
                start_bound,
                &mut postfix_path,
                postfix_location,
                postfix,
            )
        })
    }
}
