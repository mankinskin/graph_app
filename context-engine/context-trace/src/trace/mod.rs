pub mod cache;
pub mod child;
pub mod has_graph;
pub mod node;
pub mod pattern;
pub mod traceable;

use cache::{
    key::directed::{
        HasTokenPosition,
        down::DownPosition,
        up::{
            UpKey,
            UpPosition,
        },
    },
    new::{
        DownEdit,
        UpEdit,
    },
};
use has_graph::HasGraph;

use crate::{
    graph::vertex::{
        location::child::ChildLocation,
        pattern::pattern_width,
    },
    path::{
        GetRoleChildPath,
        accessors::{
            child::root::GraphRootChild,
            has_path::HasRolePath,
            role::{
                End,
                PathRole,
                Start,
            },
        },
        mutators::move_path::key::TokenPosition,
        structs::rooted::{
            index_range::IndexRangePath,
            role_path::{
                IndexEndPath,
                IndexStartPath,
            },
        },
    },
    trace::cache::{
        TraceCache,
        key::directed::{
            DirectedKey,
            down::DownKey,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum StateDirection {
    BottomUp,
    TopDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BottomUp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopDown;
// - trace
//      - End paths: top down
//      - Start paths: bottom up
// - keys are relative to the start index
pub trait TraceRolePath<Role: PathRole>:
    GetRoleChildPath + HasRolePath<Role> + GraphRootChild<Role>
{
}
impl<
    Role: PathRole,
    P: GetRoleChildPath + HasRolePath<Role> + GraphRootChild<Role>,
> TraceRolePath<Role> for P
{
}
//
// o ->x o ->x o ->x root x-> o x-> o
//
// - Start Up Key: (start, start.width())
// - Up Segment Key: (segment, pos in segment)
// - Root Up Key: (root, entry pos in root)
// - Root Down Key: (root, exit pos in root)
// - Down Segment Key: (segment, exit pos in root)
//
//       Root
//
//     /^        >\
//   Segment    Segment
//    /^           >\
// Start         End
//
pub trait TraceRole<Role: PathRole> {
    type Prev;
    fn trace_sub_path<P: TraceRolePath<Role>>(
        &mut self,
        path: &P,
        prev: Self::Prev,
        add_edges: bool,
    );
}
impl<'a, G: HasGraph> TraceRole<Start> for TraceContext<'a, G> {
    type Prev = ();
    fn trace_sub_path<P: TraceRolePath<Start>>(
        &mut self,
        path: &P,
        _prev: Self::Prev,
        add_edges: bool,
    ) {
        let graph = self.trav.graph();
        //let root_exit = path.role_root_child_location();
        let mut iter = path.raw_child_path().into_iter();
        iter.next().map(|loc| {
            let key = BottomUp::build_key(&graph, 0.into(), loc);
            iter.fold(key, |prev, loc| {
                let key = BottomUp::build_key(&graph, *prev.pos(), loc);
                let new = UpEdit {
                    target: key.clone(),
                    location: loc.clone(),
                };
                self.cache.add_state(new, add_edges);
                key
            })
        });

        //.unwrap_or_else(|| {
        //    let loc = path.role_root_child_location();
        //    UpKey {
        //        index: loc.parent,
        //        pos: graph.expect_child_offset(&loc).into(),
        //    }
        //})
    }
}
impl<'a, G: HasGraph> TraceRole<End> for TraceContext<'a, G> {
    type Prev = RoleTraceKey<End>;
    //fn trace_role_root<P: TraceRolePath<End>>(
    //    &mut self,
    //    _target: RoleTraceKey<End>,
    //    _path: &P,
    //    _add_edges: bool,
    //) {
    //  let location = path.role_root_child_location();
    //  let new = DownEdit {
    //      target,
    //      prev,
    //      location,
    //  };
    //  self.cache.add_state(new, add_edges);
    //}
    fn trace_sub_path<P: TraceRolePath<End>>(
        &mut self,
        path: &P,
        prev_key: Self::Prev,
        add_edges: bool,
    ) {
        let graph = self.trav.graph();
        //let root_exit = path.role_root_child_location();
        path.raw_child_path()
            .into_iter()
            .fold(prev_key, |prev, loc| {
                let key = TopDown::build_key(&graph, *prev.pos(), loc);
                let new = DownEdit {
                    target: key.clone(),
                    prev,
                    location: loc.clone(),
                };
                self.cache.add_state(new, add_edges);
                key
            });
    }
}
#[derive(Debug)]
pub struct TraceContext<'a, G: HasGraph> {
    pub trav: &'a G,
    pub cache: &'a mut TraceCache,
}
impl<'a, G: HasGraph> TraceContext<'a, G> {
    pub fn trace_postfix_path(
        &mut self,
        path: &IndexStartPath,
        root_up_key: RoleTraceKey<Start>,
        add_edges: bool,
    ) {
        TraceRole::<Start>::trace_sub_path(self, path, (), add_edges);
        let location = path.role_root_child_location::<Start>();
        let new = UpEdit {
            target: root_up_key.clone(),
            location,
        };
        self.cache.add_state(new, add_edges);
    }
    pub fn trace_prefix_path(
        &mut self,
        path: &IndexEndPath,
        add_edges: bool,
    ) {
        let exit_key = self.skip_key(path, 0, 0.into());
        TraceRole::<End>::trace_sub_path(self, path, exit_key, add_edges);
    }
    pub fn trace_range_path(
        &mut self,
        path: &IndexRangePath,
        root_up_key: RoleTraceKey<Start>,
        add_edges: bool,
    ) {
        TraceRole::<Start>::trace_sub_path(self, path, (), add_edges);
        let location = path.role_root_child_location::<Start>();
        let new = UpEdit {
            target: root_up_key.clone(),
            location,
        };
        self.cache.add_state(new, add_edges);

        let root_entry = path.role_root_child_location::<Start>();
        let exit_key =
            self.skip_key(path, root_entry.sub_index, root_up_key.pos);

        //TraceRole::<End>::trace_role_root(self, prev_key, path, add_edges);
        TraceRole::<End>::trace_sub_path(self, path, exit_key, add_edges);
    }
    fn skip_key<P: GraphRootChild<End>>(
        &mut self,
        path: &P,
        root_entry: usize, // sub index
        root_up_pos: UpPosition,
    ) -> RoleTraceKey<End> {
        let graph = self.trav.graph();
        let root_exit = path.role_root_child_location::<End>();

        let pattern = graph.expect_pattern_at(root_exit.clone());
        let root_down_pos = root_up_pos.0
            + pattern
                .get(root_entry + 1..root_exit.sub_index)
                .map(pattern_width)
                .unwrap_or_default();

        DownKey::new(
            *graph.expect_child_at(root_exit.clone()),
            root_down_pos.into(),
        )
    }
}

pub trait TraceKey: HasTokenPosition + Clone + Into<DirectedKey> {}
impl<T: HasTokenPosition + Clone + Into<DirectedKey>> TraceKey for T {}

pub type RoleTraceKey<Role> =
    <<Role as PathRole>::Direction as TraceDirection>::Key;
pub trait TraceDirection {
    type Opposite: TraceDirection;
    type Key: TraceKey;
    fn build_key<G: HasGraph>(
        trav: &G,
        last_pos: TokenPosition,
        location: &ChildLocation,
    ) -> Self::Key;
    fn root_target_key() {}
}

impl TraceDirection for BottomUp {
    type Opposite = TopDown;
    type Key = UpKey;
    fn build_key<G: HasGraph>(
        trav: &G,
        last_pos: TokenPosition,
        location: &ChildLocation,
    ) -> Self::Key {
        let graph = trav.graph();
        let delta = graph.expect_child_offset(location);
        UpKey {
            index: location.parent,
            pos: UpPosition::from(last_pos + delta),
        }
    }
}

impl TraceDirection for TopDown {
    type Opposite = BottomUp;
    type Key = DownKey;
    fn build_key<G: HasGraph>(
        trav: &G,
        last_pos: TokenPosition,
        location: &ChildLocation,
    ) -> Self::Key {
        let graph = trav.graph();
        let index = *graph.expect_child_at(location);
        let delta = graph.expect_child_offset(location);
        DownKey {
            index,
            pos: DownPosition::from(last_pos + delta),
        }
    }
}

//#[derive(Debug)]
//pub struct KeyIterator<'a, G: HasGraph, Role: PathRole> {
//    pub trav: &'a G,
//    pub iter: <&'a Vec<ChildLocation> as IntoIterator>::IntoIter,
//    pub last_pos: TokenPosition,
//    pub _ty: std::marker::PhantomData<Role>,
//}
//impl<'a, G: HasGraph, Role: PathRole> KeyIterator<'a, G, Role> {
//    pub fn new<P: TraceRolePath<Role> + 'a>(
//        trav: &'a G,
//        start_pos: impl Into<TokenPosition>,
//        path: &'a P,
//    ) -> Self {
//        Self {
//            trav,
//            last_pos: start_pos.into(),
//            iter: path.raw_child_path::<Role>().into_iter(),
//            _ty: Default::default(),
//        }
//    }
//}
//impl<'a, G: HasGraph, Role: PathRole> Iterator for KeyIterator<'a, G, Role> {
//    type Item = <Role::Direction as TraceDirection>::Key;
//    fn next(&mut self) -> Option<Self::Item> {
//        match self.iter.next() {
//            Some(loc) => Some({
//                let graph = self.trav.graph();
//                let delta = graph.expect_child_offset(loc);
//                let key = Role::Direction::build_key(
//                    &graph,
//                    self.last_pos,
//                    delta,
//                    loc,
//                );
//                self.last_pos = *key.pos();
//                key
//            }),
//            None => None,
//        }
//    }
//}
