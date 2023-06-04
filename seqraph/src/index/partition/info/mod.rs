use crate::*;

pub mod pattern;
pub use pattern::*;
pub mod range;
pub use range::*;
pub mod border;
pub use border::*;

#[derive(Debug, Clone, Copy)]
pub struct BundlingContext<'p> {
    pub patterns: &'p ChildPatterns,
    //cache: &'p SplitCache,
    pub index: Child,
}
pub trait AsBundlingContext<'p> {
    fn as_bundling_context<'t>(&'t self) -> BundlingContext<'t> where Self: 't, 'p: 't;
}
impl<'p> AsBundlingContext<'p> for BundlingContext<'p> {
    fn as_bundling_context<'t>(&'t self) -> BundlingContext<'t> where Self: 't, 'p: 't {
        *self
    }
}
impl<'p, P: Borrow<JoinContext<'p>>> AsBundlingContext<'p> for P {
    fn as_bundling_context<'t>(&'t self) -> BundlingContext<'t>
    where Self: 't, 'p: 't {
        BundlingContext {
            patterns: self.borrow().patterns(),
            //cache: self.borrow().cache,
            index: self.borrow().index,
        }
    }
}
impl<'p, P: Borrow<ChildPatterns>, C: Borrow<SplitCache>, I: Borrow<Child>> AsBundlingContext<'p> for (P, C, I) {
    fn as_bundling_context<'t>(&'t self) -> BundlingContext<'t>
    where Self: 't, 'p: 't {
        BundlingContext {
            patterns: self.0.borrow(),
            //cache: self.1.borrow(),
            index: *self.2.borrow(),
        }
    }
}

pub fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (
        NonZeroUsize::new(l).unwrap(),
        NonZeroUsize::new(r).unwrap(),
    )
}
#[cfg(tests)]
mod tests {
    fn first_partition() {

    }
    fn inner_partition() {
        let cache = SplitCache {
            entries: HashMap::from([]),
            leaves: vec![],
        };
        let patterns = vec![

        ];
        let (lo, ro) = to_non_zero_range(1, 3);
        let (ls, rs) = range_splits(&patterns, (lo, ro));
        let (l, r) = ((&lo, ls), (&ro, rs)); 
        let bundle = (l, r).info_bundle();
    }
    fn last_partition() {

    }
}