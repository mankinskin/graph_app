use eframe::egui::{self, pos2, vec2, Frame, Pos2, Response, Shape, Stroke, Style, Ui, Vec2, Window, Area, Rect};
#[allow(unused)]
use petgraph::{
    visit::EdgeRef,
    graph::DiGraph,
};
use seqraph::{
    hypergraph::{Child, Hypergraph, VertexData, VertexIndex, VertexKey, *},
    token::{Token, Tokenize},
};
use std::collections::{
    HashMap,
};
use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Graph,
    Nested,
}
impl Default for Layout {
    fn default() -> Self {
        Self::Graph        
    }
}
#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Graph {
    #[allow(unused)]
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Hypergraph<char>,
    pub layout: Layout,
    #[cfg_attr(feature = "persistence", serde(skip))]
    vis: GraphVis,
}
impl Graph {
    fn build_hypergraph() -> Hypergraph<char> {
        let mut g = Hypergraph::new();
        if let [a, b, c, d] = g.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])[..]
        {
            let ab = g.insert_pattern([a, b]);
            let bc = g.insert_pattern([b, c]);
            let cd = g.insert_pattern([c, d]);
            let abc = g.insert_patterns([[ab, c], [a, bc]]);
            let bcd = g.insert_patterns([[bc, d], [b, cd]]);
            let _abcd = g.insert_patterns([[abc, d], [a, bcd]]);
        } else {
            panic!("Inserting tokens failed!");
        }
        g
    }
    pub fn new() -> Self {
        let graph = Self::build_hypergraph();
        Self {
            vis: GraphVis::new(&graph),
            graph,
            layout: Default::default()
        }
    }
    pub fn show(&mut self, ui: &mut Ui) {
        self.vis.show(ui)
    }
}
pub struct GraphVis {
    graph: petgraph::graph::Graph<Node, (), petgraph::Directed>,
}
impl GraphVis {
    pub fn new(g: &Hypergraph<char>) -> Self {
        // todo reuse names in nodes
        let pg = g.to_petgraph();
        let graph =
            pg.map(|_idx, (key, node)| Node::new(&g, &key, node),
            |_idx, _p| ()
        );
        let nodes: Vec<_> = g
            .vertex_iter()
            .map(|(key, node)| Node::new(&g, key, node))
            .collect();
        //let _edges: Vec<_> = g.vertex_iter().flat_map(|(key, node)| {
        //    let id = g.get_index_by_key(key);
        //    let child_patterns = node.
        //    let name = g.key_data_string(key, node);
        //    let text = format!("key: {:#?}", key);
        //    Node::new(id, name, text)
        //}).collect();
        Self {
            graph,
        }
    }
    pub fn edge_tip(ui: &mut Ui, source: &Pos2, target: &Pos2, size: f32) {
        
        let angle = (*target - *source).angle();
        let points = IntoIterator::into_iter(
            [
                Vec2::new(0.0, 0.0),
                Vec2::angled(angle - 0.25*PI),
                Vec2::angled(angle + 0.25*PI)
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
    pub fn edge(ui: &mut Ui, source: &Pos2, target: &Pos2) {
        ui.painter().add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
        Self::edge_tip(ui, source, target, 10.0);
    }
    fn border_intersection_point(rect: &Rect, p: &Pos2) -> Pos2 {
        let p = *p;
        let c = rect.center();
        let v = p - c;
        let s = v.y/v.x;
        let h = rect.height();
        let w = rect.width();
        c + if -h/2.0 <= s*w/2.0 && s*w/2.0 <= h/2.0 {
            // intersects side
            if p.x > c.x {
                // right
                vec2(w/2.0, w/2.0*s)
            } else {
                // left
                vec2(-w/2.0, -w/2.0*s)
            }
        } else {
            // intersects top or bottom
            if p.y > c.y {
                // top
                vec2(h/(2.0*s), h/2.0)
            } else {
                // bottom
                vec2(-h/(2.0*s), -h/2.0)
            }
        }
    }
    pub fn show(&mut self, ui: &mut Ui) {
        //println!("{}", self.graph.vertex_count());
        let rects: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| {
                let response = node.clone().show(ui).unwrap();
                (idx, response.rect)
            })
            .collect();
        self.graph.edge_references()
            .for_each(|edge| {
                let a_pos = rects.get(&edge.source()).expect("No position for edge endpoint.").center();
                let b = rects.get(&edge.target()).expect("No position for edge endpoint.");

                let p = Self::border_intersection_point(&b, &a_pos);
                Self::edge(ui, &a_pos, &p);
            });
        //let pos1 = node1.rect.center();
        //let pos2 = node2.rect.center();
        //let _edge = self.edge(ui, pos1, pos2);
    }
}
#[derive(Clone)]
struct ChildVis {
    name: String,
    child: Child,
}
impl std::ops::Deref for ChildVis {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl ChildVis {
    fn new<T: Tokenize + std::fmt::Display>(graph: &Hypergraph<T>, child: Child) -> Self {
        let name = graph.index_string(child.get_index());
        Self { name, child }
    }
}
#[derive(Clone)]
struct PatternVis {
    width: TokenPosition,
    pattern: Vec<ChildVis>,
}
impl PatternVis {
    fn new(pattern: Vec<ChildVis>) -> Self {
        let width = pattern_width((&pattern).iter().map(|c| &c.child).collect::<Vec<_>>());
        Self { width, pattern }
    }
}
#[derive(Clone)]
pub struct Node {
    id: VertexIndex,
    name: String,
    data: VertexData,
    child_patterns: ChildPatternsVis,
}
impl std::ops::Deref for Node {
    type Target = VertexData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
type ChildPatternsVis = Vec<(PatternId, PatternVis)>;
#[allow(unused)]
const UNIT_WIDTH: f32 = 5.0;
impl Node {
    pub fn new<T: Tokenize + std::fmt::Display>(
        graph: &Hypergraph<T>,
        key: &VertexKey<T>,
        data: &VertexData,
    ) -> Self {
        let id = graph.expect_index_by_key(key);
        let name = graph.key_data_string(key, data);
        //let text = format!("key: {:#?}", key);
        let child_patterns = Self::child_patterns_vis(graph, data);
        Self {
            id,
            name,
            data: data.clone(),
            child_patterns,
        }
    }
    fn child_patterns_vis<T: Tokenize + std::fmt::Display>(
        graph: &Hypergraph<T>,
        data: &VertexData,
    ) -> ChildPatternsVis {
        data.get_children()
            .iter()
            .map(|(&id, pat)| {
                (
                    id,
                    PatternVis::new(
                        pat.iter()
                            .map(|c| ChildVis::new(graph, c.clone()))
                            .collect(),
                    ),
                )
            })
            .collect()
    }
    pub fn child_patterns(&self, ui: &mut Ui) -> Vec<Vec<Response>> {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.child_patterns
                .iter()
                .map(|(_pid, cpat)| self.pattern(ui, cpat))
                .collect()
        })
        .inner
    }
    fn pattern(&self, ui: &mut Ui, pat: &PatternVis) -> Vec<Response> {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            //ui.set_min_width(UNIT_WIDTH * pat.width as f32);
            pat.pattern
                .iter()
                .map(|child| self.child_index(ui, child))
                .collect()
        })
        .inner
    }
    fn child_index(&self, ui: &mut Ui, child: &ChildVis) -> Response {
        Frame::group(&Style::default())
            .show(ui, |ui| {
                //ui.set_min_width(UNIT_WIDTH * child.width as f32);
                ui.label(format!("{}", child.name))
            })
            .response
    }
    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        Window::new(&self.name)
            //.auto_sized()
            .resizable(false)
            .collapsible(false)
            .min_width(100.0 * UNIT_WIDTH * self.width as f32)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                self.child_patterns(ui)
            })
            .map(|ir| ir.response)
    }
}
