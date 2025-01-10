pub mod context;
pub mod merge;
pub mod kind;

//pub trait ToNodeJoinContext<'p> {
//    fn to_node_join_context<'t>(self) -> NodeJoinContext<'t>
//    where
//        Self: 't,
//        'p: 't;
//}
//impl<'p> ToNodeJoinContext<'p> for NodeJoinContext<'p> {
//    fn to_node_join_context<'t>(self) -> NodeJoinContext<'t>
//    where
//        Self: 't,
//        'p: 't,
//    {
//        self
//    }
//}
