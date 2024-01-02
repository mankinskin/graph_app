use crate::shared::*;

pub trait VisitMode<K: RangeRole<Mode = Self>>:
    Debug + Clone + Copy +
    ModeChildren<K> + for<'a> ModeContext<'a>
{
    type RangeInfo: ModeRangeInfo<K>;
}
pub type RangeInfoOf<K> = <ModeOf<K> as VisitMode<K>>::RangeInfo;

impl<K: RangeRole<Mode = Self>> VisitMode<K> for Trace {
    type RangeInfo = TraceRangeInfo<K>;
}
impl<K: RangeRole<Mode = Self>> VisitMode<K> for Join
    where K::Borders: JoinBorders<K>
{
    type RangeInfo = JoinRangeInfo<K>;
}