use crate::*;
pub mod vertex;
pub use vertex::*;
pub mod split;
pub use split::*;

#[derive(Debug)]
pub struct SplitCache {
    pub entries: HashMap<VertexIndex, SplitVertexCache>,
    pub leaves: Vec<SplitKey>,
    pub root_mode: RootMode,
}
impl SplitCache {
    pub fn get(&self, key: &SplitKey) -> Option<&SplitPositionCache> {
        self.entries.get(&key.index.index())
            .and_then(|ve|
                ve.positions.get(&key.pos)
            )
    }
    pub fn get_mut(&mut self, key: &SplitKey) -> Option<&mut SplitPositionCache> {
        self.entries.get_mut(&key.index.index())
            .and_then(|ve|
                ve.positions.get_mut(&key.pos)
            )
    }
    pub fn expect(&self, key: &SplitKey) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(&mut self, key: &SplitKey) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
    pub fn get_final_split(&self, key: &SplitKey) -> Option<&FinalSplit> {
        self.get(key)
            .and_then(|e|
                e.final_split.as_ref()
            )
    }
    pub fn expect_final_split(&self, key: &SplitKey) -> &FinalSplit {
        self.expect(key).final_split.as_ref().unwrap()
    }
}

pub fn position_splits<'a>(
    patterns: impl Iterator<Item=(&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Vec<SubSplitLocation> {
    patterns
        .map(|(pid, pat)| { 
            let (sub_index, inner_offset) = <IndexBack as IndexSide<Right>>::token_offset_split(
                pat.borrow(),
                parent_offset,
            ).unwrap();
            let location = SubLocation::new(*pid, sub_index);
            SubSplitLocation {
                location,
                inner_offset,
            }
        })
        .collect()
}
pub fn cleaned_position_splits<'a>(
    patterns: impl Iterator<Item=(&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Result<Vec<SubSplitLocation>, SubLocation> {
    patterns
        .map(|(pid, pat)| { 
            let (sub_index, inner_offset) = <IndexBack as IndexSide<Right>>::token_offset_split(
                pat.borrow(),
                parent_offset,
            ).unwrap();
            let location = SubLocation::new(*pid, sub_index);
            if inner_offset.is_some() || pat.len() > 2 {
                // can't be clean
                Ok(SubSplitLocation {
                    location,
                    inner_offset
                })
            } else {
                // must be clean
                Err(location)
            }
        })
        .collect()
}