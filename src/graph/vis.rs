use crate::*;
use eframe::egui::{
    self,
    vec2,
    Frame,
    Pos2,
    Rect,
    Response,
    Shape,
    Stroke,
    Style,
    Ui,
    Vec2,
    Window,
};
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};
use seqraph::graph::vertex::Wide;
use std::collections::HashMap;
use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum Layout
{
    Graph,
    Nested,
}
impl Layout
{
    #[allow(unused)]
    pub fn is_graph(&self) -> bool
    {
        matches!(self, Self::Graph)
    }
    pub fn is_nested(&self) -> bool
    {
        matches!(self, Self::Nested)
    }
}
impl Default for Layout
{
    fn default() -> Self
    {
        Self::Graph
    }
}
#[derive(Default)]
pub struct GraphVis
{
    graph: DiGraph<NodeVis, ()>,
    pub layout: Layout,
    handle: Option<Graph>,
}
impl GraphVis
{
    pub fn set_graph(
        &mut self,
        graph: Graph,
    )
    {
        self.handle = Some(graph);
        self.update();
    }
    fn graph(&self) -> Graph
    {
        self.handle.clone().expect("GraphVis not yet initialized!")
    }
    pub fn update(&mut self) -> Option<()>
    {
        // todo reuse names in nodes
        //println!("update...");
        let pg = self.graph().read().to_petgraph();
        //println!("updating");
        let old_node_indices: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| (node.key, idx))
            .collect();
        let filtered = pg.filter_map(
            |_idx, (key, node)| {
                if node.width() <= 1
                {
                    None
                }
                else
                {
                    Some((*key, node))
                }
            },
            |_idx, e| (e.child.width() > 1).then(|| ()),
        );
        let node_indices: HashMap<_, _> = filtered
            .nodes()
            .map(|(idx, (key, _node))| (*key, idx))
            .collect();

        self.graph = filtered.map(
            |idx, (key, node)| {
                if let Some(oid) = old_node_indices.get(key)
                {
                    let old = self.graph.node_weight(*oid).unwrap();
                    NodeVis::from_old(old, &node_indices, idx, node)
                }
                else
                {
                    NodeVis::new(self.graph(), &node_indices, idx, key, node)
                }
            },
            |_idx, _e| (),
        );
        //println!("done");
        Some(())
    }
    pub fn show(
        &mut self,
        ui: &mut Ui,
    )
    {
        //println!("{}", self.graph.vertex_count());
        let rects: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| {
                let response = node.show(ui, self).unwrap();
                (idx, response.rect)
            })
            .collect();
        self.graph.edge_references().for_each(|edge| {
            let a_pos = rects
                .get(&edge.source())
                .expect("No position for edge endpoint.")
                .center();
            let b = rects
                .get(&edge.target())
                .expect("No position for edge endpoint.");

            let p = Self::border_intersection_point(b, &a_pos);
            Self::edge(ui, &a_pos, &p);
        });
    }
    pub fn edge_tip(
        ui: &mut Ui,
        source: &Pos2,
        target: &Pos2,
        size: f32,
    )
    {
        let angle = (*target - *source).angle();
        let points = IntoIterator::into_iter([
            Vec2::new(0.0, 0.0),
            Vec2::angled(angle - 0.25 * PI),
            Vec2::angled(angle + 0.25 * PI),
        ])
        .map(|p| *target - p * size)
        .collect();
        ui.painter().add(Shape::convex_polygon(
            points,
            egui::Color32::WHITE,
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
    }
    #[allow(unused)]
    pub fn edge(
        ui: &mut Ui,
        source: &Pos2,
        target: &Pos2,
    )
    {
        ui.painter().add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
        Self::edge_tip(ui, source, target, 10.0);
    }
    #[allow(clippy::many_single_char_names)]
    fn border_intersection_point(
        rect: &Rect,
        p: &Pos2,
    ) -> Pos2
    {
        let p = *p;
        let c = rect.center();
        let v = p - c;
        let s = v.y / v.x;
        let h = rect.height();
        let w = rect.width();
        c + if -h / 2.0 <= s * w / 2.0 && s * w / 2.0 <= h / 2.0
        {
            // intersects side
            if p.x > c.x
            {
                // right
                vec2(w / 2.0, w / 2.0 * s)
            }
            else
            {
                // left
                vec2(-w / 2.0, -w / 2.0 * s)
            }
        }
        else
        {
            // intersects top or bottom
            if p.y > c.y
            {
                // top
                vec2(h / (2.0 * s), h / 2.0)
            }
            else
            {
                // bottom
                vec2(-h / (2.0 * s), -h / 2.0)
            }
        }
    }
}
#[allow(unused)]
#[derive(Clone)]
pub struct NodeVis
{
    key: VertexKey<char>,
    idx: NodeIndex,
    name: String,
    data: VertexData,
    child_patterns: ChildPatternsVis,
    state: Arc<RwLock<NodeState>>,
    graph: Graph,
}
#[allow(unused)]
pub struct NodeState
{
    split_lower: usize,
    split_upper: usize,
}
impl NodeState
{
    pub fn new() -> Self
    {
        Self {
            split_lower: 1,
            split_upper: 7,
        }
    }
}
impl std::ops::Deref for NodeVis
{
    type Target = VertexData;
    fn deref(&self) -> &Self::Target
    {
        &self.data
    }
}
impl NodeVis
{
    pub fn new(
        graph: Graph,
        node_indices: &HashMap<VertexKey<char>, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey<char>,
        data: &VertexData,
    ) -> Self
    {
        Self::new_impl(
            graph,
            node_indices,
            idx,
            key,
            data,
            Arc::new(RwLock::new(NodeState::new())),
        )
    }
    pub fn from_old(
        old: &NodeVis,
        node_indices: &HashMap<VertexKey<char>, NodeIndex>,
        idx: NodeIndex,
        data: &VertexData,
    ) -> Self
    {
        Self::new_impl(
            old.graph.clone(),
            node_indices,
            idx,
            &old.key,
            data,
            old.state.clone(),
        )
    }
    pub fn new_impl(
        graph: Graph,
        node_indices: &HashMap<VertexKey<char>, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey<char>,
        data: &VertexData,
        state: Arc<RwLock<NodeState>>,
    ) -> Self
    {
        let (name, child_patterns) = {
            let graph = &*graph.read();
            let name = graph.key_data_string(key, data);
            let child_patterns =
                Self::child_patterns_vis(graph, node_indices, data);
            (name, child_patterns)
        };
        Self {
            key: *key,
            graph,
            idx,
            name,
            data: data.clone(),
            child_patterns,
            state,
        }
    }
    #[allow(unused)]
    fn state(&self) -> RwLockReadGuard<'_, NodeState>
    {
        self.state.read().unwrap()
    }
    #[allow(unused)]
    fn state_mut(&self) -> RwLockWriteGuard<'_, NodeState>
    {
        self.state.write().unwrap()
    }
    fn child_patterns_vis(
        graph: &Hypergraph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        data: &VertexData,
    ) -> ChildPatternsVis
    {
        data.get_child_patterns()
            .iter()
            .map(|(&id, pat)| {
                (
                    id,
                    PatternVis::new(
                        pat.iter()
                            .map(|c| ChildVis::new(graph, node_indices, *c))
                            .collect(),
                    ),
                )
            })
            .collect()
    }
    pub fn child_patterns(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> Response
    {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.child_patterns.iter().for_each(|(_pid, cpat)| {
                let r = self.measure_pattern(ui, cpat, gvis);
                let height = r.rect.height();
                self.pattern(ui, cpat, Some(height), gvis);
            })
        })
        .response
    }
    fn measure_pattern(
        &self,
        ui: &mut Ui,
        pat: &PatternVis,
        gvis: &GraphVis,
    ) -> Response
    {
        let old_clip_rect = ui.clip_rect();
        let old_cursor = ui.cursor();
        ui.set_clip_rect(Rect::NOTHING);
        let r = self.pattern(ui, pat, None, gvis);
        ui.set_clip_rect(old_clip_rect);
        ui.set_cursor(old_cursor);
        r
    }
    fn pattern(
        &self,
        ui: &mut Ui,
        pat: &PatternVis,
        height: Option<f32>,
        gvis: &GraphVis,
    ) -> Response
    {
        ui.horizontal(|ui| {
            if let Some(height) = height
            {
                ui.set_min_height(height);
            }
            pat.pattern.iter().for_each(|child| {
                self.child_index(ui, child, gvis);
            })
        })
        .response
    }
    fn child_index(
        &self,
        ui: &mut Ui,
        child: &ChildVis,
        gvis: &GraphVis,
    ) -> Response
    {
        Frame::group(&Style::default())
            .inner_margin(3.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                //ui.set_min_width(UNIT_WIDTH * child.width as f32);
                if gvis.layout.is_nested() && child.idx.is_some()
                {
                    assert!(child.child.width() > 1);
                    let node = gvis
                        .graph
                        .node_weight(child.idx.unwrap())
                        .expect("Invalid NodeIndex in ChildVis!");
                    node.child_patterns(ui, gvis)
                }
                else
                {
                    ui.monospace(&child.name)
                }
            })
            .response
    }
    pub fn show(
        &self,
        ui: &mut Ui,
        gvis: &GraphVis,
    ) -> Option<Response>
    {
        Window::new(&format!("{}({})", self.name, self.idx.index()))
            //Window::new(&self.name)
            .vscroll(true)
            .default_width(80.0)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                self.child_patterns(ui, gvis)
            })
            .map(|ir| ir.response)
    }
}
#[derive(Clone)]
struct ChildVis
{
    name: String,
    child: Child,
    idx: Option<NodeIndex>,
}
impl std::ops::Deref for ChildVis
{
    type Target = Child;
    fn deref(&self) -> &Self::Target
    {
        &self.child
    }
}
impl ChildVis
{
    fn new(
        graph: &Hypergraph,
        node_indices: &HashMap<VertexKey, NodeIndex>,
        child: Child,
    ) -> Self
    {
        let key = graph.expect_vertex_key(child.index);
        let name = graph.index_string(child.get_index());
        let idx = node_indices.get(key).cloned();
        Self { name, child, idx }
    }
}
#[derive(Clone)]
struct PatternVis
{
    pattern: Vec<ChildVis>,
}
impl PatternVis
{
    fn new(pattern: Vec<ChildVis>) -> Self
    {
        Self { pattern }
    }
}
type ChildPatternsVis = Vec<(PatternId, PatternVis)>;
