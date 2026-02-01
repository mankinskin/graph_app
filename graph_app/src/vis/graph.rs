use context_trace::{
    graph::vertex::{
        has_vertex_key::HasVertexKey,
        key::VertexKey,
        location::pattern::PatternLocation,
        wide::Wide,
    },
    End,
    HashMap,
    IndexRangePath,
    IndexRoot,
    RolePath,
    Start,
};
use eframe::egui::{
    self,
    vec2,
    Color32,
    Pos2,
    Rect,
    Shape,
    Stroke,
    Ui,
    Vec2,
};
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};

use std::f32::consts::PI;

use super::node::NodeVis;
use crate::{
    graph::Graph,
    vis::{
        layout::GraphLayout,
        node::SelectionState,
    },
};

/// Response from showing the graph
#[derive(Debug, Default)]
pub struct GraphResponse {
    /// The node that was clicked, if any
    pub clicked_node: Option<VertexKey>,
    /// Whether the background was clicked (no node clicked)
    pub background_clicked: bool,
}

#[derive(Debug)]
pub struct GraphVis {
    graph: DiGraph<NodeVis, ()>,
    handle: Option<Graph>,
    layout: GraphLayout,
    /// Flag to indicate the graph needs to be rebuilt
    dirty: bool,
    /// Generation counter - increments on each rebuild to reset window positions
    generation: usize,
    /// Zoom level (1.0 = 100%)
    zoom: f32,
    /// Pan offset
    pan: Vec2,
}

impl Default for GraphVis {
    fn default() -> Self {
        Self {
            graph: DiGraph::new(),
            handle: None,
            layout: GraphLayout::default(),
            dirty: false,
            generation: 0,
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}
#[derive(Debug)]
#[allow(unused)]
pub enum UpdateError {
    NoRecordingStream,
    Stream(rerun::RecordingStreamError),
    NotInitialized,
}
use UpdateError::*;
impl GraphVis {
    fn update_rerun(
        &mut self,
        handle: &Graph,
    ) -> Result<(), UpdateError> {
        let rec = handle.rec.as_ref().ok_or(NoRecordingStream)?;
        rec.log_static("/graph", &self.layout.re_nodes())
            .map_err(Stream)?;

        rec.log_static("/graph", &self.layout.re_edges())
            .map_err(Stream)?;
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), UpdateError> {
        let handle = self.graph().ok_or(NotInitialized)?;
        let cg = handle.read();

        let pg = cg.to_petgraph().filter_map_owned(
            |_idx, (_index, node)| {
                if node.data.width() <= 1 {
                    None
                } else {
                    Some((node.data.vertex_key(), node))
                }
            },
            |_idx, e| (e.token.width() > 1).then_some(()),
        );

        // Save manually moved positions before regenerating
        let saved_positions: HashMap<VertexKey, Pos2> = self
            .graph
            .node_weights()
            .filter(|n| n.manually_moved)
            .map(|n| (n.key, n.world_pos))
            .collect();

        // Increment generation to reset window positions
        self.generation += 1;

        // Regenerate layout
        self.layout = GraphLayout::generate(&cg, pg);
        // Clear the old graph to force rebuild in update_egui
        self.graph = DiGraph::new();

        // Try to update rerun, but don't fail if it's not available
        let _ = self.update_rerun(&handle);
        self.update_egui(&handle, &saved_positions);
        Ok(())
    }
    fn update_egui(
        &mut self,
        handle: &Graph,
        saved_positions: &HashMap<VertexKey, Pos2>,
    ) {
        let generation = self.generation;
        if !self.initialized() {
            self.graph = self.layout.graph.clone().map_owned(
                |i, (k, n)| {
                    // Use saved position if this node was manually moved, otherwise use layout position
                    let (pos, manually_moved) =
                        if let Some(&saved_pos) = saved_positions.get(&k) {
                            (saved_pos, true)
                        } else {
                            let pos = self
                                .layout
                                .positions
                                .get(&i)
                                .copied()
                                .unwrap_or_default();
                            (pos, false)
                        };
                    let mut vis = NodeVis::new(
                        handle.clone(),
                        i,
                        &k,
                        &n.data,
                        pos,
                        generation,
                    );
                    vis.manually_moved = manually_moved;
                    vis
                },
                |_, e| e,
            );
        }
        for (_key, (i, node)) in self.layout.nodes.iter_mut() {
            // Use saved position if this node was manually moved
            let (pos, manually_moved) = if let Some(&saved_pos) =
                saved_positions.get(_key)
            {
                (saved_pos, true)
            } else {
                let pos =
                    self.layout.positions.get(i).copied().unwrap_or_default();
                (pos, false)
            };
            if let Some(old) = self.graph.node_weight_mut(*i) {
                *old = NodeVis::from_old(old, *i, &node.data);
            } else {
                let mut vis = NodeVis::new(
                    handle.clone(),
                    *i,
                    _key,
                    &node.data,
                    pos,
                    generation,
                );
                vis.manually_moved = manually_moved;
                *i = self.graph.add_node(vis);
            };
        }
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(
        &self,
        world_pos: Pos2,
        viewport_min: Pos2,
    ) -> Pos2 {
        viewport_min + (world_pos.to_vec2() * self.zoom) + self.pan
    }

    /// Convert screen coordinates to world coordinates
    #[allow(unused)]
    fn screen_to_world(
        &self,
        screen_pos: Pos2,
        viewport_min: Pos2,
    ) -> Pos2 {
        ((screen_pos - viewport_min - self.pan) / self.zoom).to_pos2()
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) -> GraphResponse {
        let mut response = GraphResponse::default();

        // Update if not initialized OR if marked dirty
        if !self.initialized() || self.dirty {
            if let Err(err) = self.update() {
                println!("Error updating graph: {:?}", err);
            }
            self.dirty = false;
        }
        let _events = self.poll_events();

        // Get the available rect for the graph viewport
        let viewport_rect = ui.available_rect_before_wrap();

        // Allocate the entire viewport area for our canvas
        let canvas_response =
            ui.allocate_rect(viewport_rect, egui::Sense::click_and_drag());

        // Draw background
        let painter = ui.painter().with_clip_rect(viewport_rect);
        painter.rect_filled(viewport_rect, 0.0, Color32::from_rgb(25, 28, 32));

        // Draw subtle grid
        self.draw_grid(&painter, viewport_rect);

        // Handle zoom with scroll wheel
        let hover_pos = ui.input(|i| i.pointer.hover_pos());
        let hovering_graph = hover_pos
            .map(|p| viewport_rect.contains(p))
            .unwrap_or(false);

        if hovering_graph {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = 1.0 + scroll_delta * 0.001;
                let new_zoom = (self.zoom * zoom_factor).clamp(0.1, 5.0);

                // Zoom towards mouse position
                if let Some(mouse_pos) = hover_pos {
                    let mouse_rel = mouse_pos - viewport_rect.min - self.pan;
                    let zoom_change = new_zoom / self.zoom;
                    self.pan -= mouse_rel * (zoom_change - 1.0);
                }

                self.zoom = new_zoom;
            }
        }

        // Handle pan with middle mouse button drag
        if canvas_response.dragged_by(egui::PointerButton::Middle) {
            let delta = ui.input(|i| i.pointer.delta());
            self.pan += delta;
        }

        // Show zoom level and pan info
        painter.text(
            viewport_rect.right_top() + vec2(-10.0, 10.0),
            egui::Align2::RIGHT_TOP,
            format!("{:.0}%", self.zoom * 100.0),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(150, 150, 150),
        );

        // Cache zoom and pan for use in loops
        let zoom = self.zoom;
        let pan = self.pan;
        let viewport_min = viewport_rect.min;

        // First pass: render nodes and collect their screen rects
        let mut node_screen_rects: HashMap<NodeIndex, Rect> =
            HashMap::default();
        let mut dragged_node: Option<NodeIndex> = None;
        let mut clicked_node: Option<VertexKey> = None;

        for (idx, node) in self.graph.nodes_mut() {
            let screen_pos =
                viewport_min + (node.world_pos.to_vec2() * zoom) + pan;

            if let Some(node_response) =
                node.show(ui, screen_pos, zoom, viewport_rect)
            {
                node_screen_rects.insert(idx, node_response.rect);

                if node_response
                    .response
                    .dragged_by(egui::PointerButton::Primary)
                {
                    dragged_node = Some(idx);
                }

                // Detect click (clicked = was pressed and released)
                if node_response.response.clicked() {
                    clicked_node = Some(node.key);
                }
            }
        }

        // Second pass: draw edges ON TOP of source nodes
        // Source = parent node (has child patterns)
        // Target = child node (pointed to by arrow)
        // Edge starts FROM the child's frame in the source/parent node's patterns
        // Edge ends AT the target/child node
        let clipped_painter = ui.painter().with_clip_rect(viewport_rect);

        for edge in self.graph.edge_references() {
            let source_idx = edge.source(); // Parent node
            let target_idx = edge.target(); // Child node

            if let Some((source_rect, target_rect)) = node_screen_rects
                .get(&source_idx)
                .zip(node_screen_rects.get(&target_idx))
            {
                // Get the target/child node's vertex index to find it in source's child patterns
                let target_vertex_idx = self
                    .graph
                    .node_weight(target_idx)
                    .map(|n| *n.data.to_child().index);

                // Get all child rects for this target from source node (same child can appear multiple times)
                let child_rects: Vec<Rect> =
                    if let Some(tgt_v_idx) = target_vertex_idx {
                        self.graph
                            .node_weight(source_idx)
                            .and_then(|node| node.child_rects.get(&tgt_v_idx))
                            .map(|rects| rects.clone())
                            .unwrap_or_default()
                    } else {
                        vec![]
                    };

                // End point: at target/child node
                let target_center = target_rect.center();

                if child_rects.is_empty() {
                    // Fallback: draw edge from source center to target
                    let start = Self::border_intersection_point(
                        source_rect,
                        &target_center,
                    );
                    let end = Self::border_intersection_point(
                        target_rect,
                        &source_rect.center(),
                    );
                    Self::edge_clipped(&clipped_painter, &start, &end, zoom);
                } else {
                    // Draw an edge from each occurrence of the child in source's patterns
                    for child_rect in &child_rects {
                        let start = Self::border_intersection_point(
                            child_rect,
                            &target_center,
                        );
                        let end = Self::border_intersection_point(
                            target_rect,
                            &child_rect.center(),
                        );
                        Self::edge_clipped(
                            &clipped_painter,
                            &start,
                            &end,
                            zoom,
                        );
                    }
                }
            }
        }

        // Handle node dragging - update world position
        if let Some(idx) = dragged_node {
            let delta = ui.input(|i| i.pointer.delta());
            if let Some(node) = self.graph.node_weight_mut(idx) {
                // Convert screen delta to world delta
                let world_delta = delta / zoom;
                node.world_pos += world_delta;
                // Mark as manually moved
                node.manually_moved = true;
            }
        }

        // Set clicked node in response
        response.clicked_node = clicked_node;

        // Detect background click (canvas clicked but no node was clicked)
        if canvas_response.clicked() && clicked_node.is_none() {
            response.background_clicked = true;
        }

        response
    }

    fn draw_grid(
        &self,
        painter: &egui::Painter,
        viewport_rect: Rect,
    ) {
        let grid_spacing = 50.0 * self.zoom;
        if grid_spacing < 10.0 {
            return; // Don't draw grid when too zoomed out
        }

        let grid_color = Color32::from_rgba_unmultiplied(255, 255, 255, 8);

        // Calculate grid offset based on pan
        let offset_x = self.pan.x % grid_spacing;
        let offset_y = self.pan.y % grid_spacing;

        // Vertical lines
        let mut x = viewport_rect.min.x + offset_x;
        while x < viewport_rect.max.x {
            painter.line_segment(
                [
                    Pos2::new(x, viewport_rect.min.y),
                    Pos2::new(x, viewport_rect.max.y),
                ],
                Stroke::new(1.0, grid_color),
            );
            x += grid_spacing;
        }

        // Horizontal lines
        let mut y = viewport_rect.min.y + offset_y;
        while y < viewport_rect.max.y {
            painter.line_segment(
                [
                    Pos2::new(viewport_rect.min.x, y),
                    Pos2::new(viewport_rect.max.x, y),
                ],
                Stroke::new(1.0, grid_color),
            );
            y += grid_spacing;
        }
    }

    fn initialized(&self) -> bool {
        self.graph.node_count() > 0 && self.handle.is_some()
    }

    /// Mark the graph visualization as needing a rebuild
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn new(graph: Graph) -> Self {
        Self {
            graph: DiGraph::new(),
            handle: Some(graph),
            layout: GraphLayout::default(),
            dirty: false,
            generation: 0,
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
    fn graph(&self) -> Option<Graph> {
        self.handle.clone()
    }
    pub fn edge_tip(
        ui: &mut Ui,
        source: &Pos2,
        target: &Pos2,
        size: f32,
    ) {
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
    fn edge_tip_clipped(
        painter: &egui::Painter,
        source: &Pos2,
        target: &Pos2,
        size: f32,
        zoom: f32,
    ) {
        let angle = (*target - *source).angle();
        let points = IntoIterator::into_iter([
            Vec2::new(0.0, 0.0),
            Vec2::angled(angle - 0.25 * PI),
            Vec2::angled(angle + 0.25 * PI),
        ])
        .map(|p| *target - p * size)
        .collect();
        painter.add(Shape::convex_polygon(
            points,
            egui::Color32::WHITE,
            Stroke::new(1.0 * zoom, egui::Color32::WHITE),
        ));
    }
    #[allow(unused)]
    pub fn edge(
        ui: &mut Ui,
        source: &Pos2,
        target: &Pos2,
    ) {
        ui.painter().add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
        Self::edge_tip(ui, source, target, 10.0);
    }
    fn edge_clipped(
        painter: &egui::Painter,
        source: &Pos2,
        target: &Pos2,
        zoom: f32,
    ) {
        painter.add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0 * zoom, egui::Color32::WHITE),
        ));
        Self::edge_tip_clipped(painter, source, target, 10.0 * zoom, zoom);
    }
    #[allow(clippy::many_single_char_names)]
    fn border_intersection_point(
        rect: &Rect,
        p: &Pos2,
    ) -> Pos2 {
        let p = *p;
        let c = rect.center();
        let v = p - c;
        let s = v.y / v.x;
        let h = rect.height();
        let w = rect.width();
        c + if -h / 2.0 <= s * w / 2.0 && s * w / 2.0 <= h / 2.0 {
            // intersects side
            if p.x > c.x {
                // right
                vec2(w / 2.0, w / 2.0 * s)
            } else {
                // left
                vec2(-w / 2.0, -w / 2.0 * s)
            }
        } else {
            // intersects top or bottom
            if p.y > c.y {
                // top
                vec2(h / (2.0 * s), h / 2.0)
            } else {
                // bottom
                vec2(-h / (2.0 * s), -h / 2.0)
            }
        }
    }
    pub fn poll_events(&self) -> Vec<tracing_egui::LogEvent> {
        //println!("polling..");
        tracing_egui::poll_events().drain(..).collect()
    }
}
