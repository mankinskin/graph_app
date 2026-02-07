use std::{
    collections::HashMap,
    ops::Range,
};

use context_trace::{
    graph::vertex::{
        data::VertexData,
        key::VertexKey,
        pattern::id::PatternId,
    },
    IndexRangePath,
};
use eframe::{
    egui::{
        self,
        Color32,
        CornerRadius,
        Pos2,
        Rect,
        Response,
        Sense,
        Stroke,
        StrokeKind,
        Ui,
        Vec2,
    },
    epaint::FontId,
};
use indexmap::IndexMap;
use petgraph::graph::NodeIndex;

use crate::{
    graph::Graph,
    vis::pattern::ChildPatternsVis,
};

pub(crate) struct NodeResponse {
    pub(crate) response: Response,
    pub(crate) rect: Rect,
    pub(crate) ranges: IndexMap<PatternId, Range<usize>>,
}
#[allow(unused)]
#[derive(Clone, Debug)]
pub(crate) struct SelectionState {
    pub(crate) pattern_id: PatternId,
    pub(crate) range: Range<usize>,
    pub(crate) trace: IndexRangePath,
}
#[allow(unused)]
#[derive(Clone, Debug)]
pub(crate) struct NodeVis {
    pub(crate) key: VertexKey,
    idx: NodeIndex,
    pub(crate) name: String,
    pub(crate) data: VertexData,
    /// Position in world coordinates
    pub(crate) world_pos: Pos2,
    /// Whether this node was manually moved by the user
    pub(crate) manually_moved: bool,
    pub(crate) child_patterns: ChildPatternsVis,
    graph: Graph,
    pub(crate) selected_range: Option<SelectionState>,
    /// Generation counter for unique IDs
    generation: usize,
    /// Cached size from last render
    pub(crate) cached_size: Vec2,
    /// Map from child vertex index to its screen rects (updated during render)
    pub(crate) child_rects: HashMap<usize, Vec<Rect>>,
}

impl std::ops::Deref for NodeVis {
    type Target = VertexData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl NodeVis {
    pub(crate) fn new(
        graph: Graph,
        idx: NodeIndex,
        key: &VertexKey,
        data: &VertexData,
        world_pos: Pos2,
        generation: usize,
    ) -> Self {
        Self::new_impl(graph, idx, key, data, world_pos, None, generation)
    }
    pub(crate) fn from_old(
        old: &Self,
        idx: NodeIndex,
        data: &VertexData,
    ) -> Self {
        let mut new = Self::new_impl(
            old.graph.clone(),
            idx,
            &old.key,
            data,
            old.world_pos,
            old.selected_range.clone(),
            old.generation,
        );
        new.manually_moved = old.manually_moved;
        new
    }
    pub(crate) fn new_impl(
        graph: Graph,
        idx: NodeIndex,
        key: &VertexKey,
        data: &VertexData,
        world_pos: Pos2,
        selected_range: Option<SelectionState>,
        generation: usize,
    ) -> Self {
        let (name, child_patterns) = {
            let graph = &**graph.read();
            let name = graph.vertex_data_string(data.clone());
            let child_patterns = ChildPatternsVis::new(graph, data);
            (name, child_patterns)
        };
        Self {
            key: *key,
            graph,
            idx,
            name,
            data: data.clone(),
            world_pos,
            manually_moved: false,
            child_patterns,
            selected_range,
            generation,
            cached_size: Vec2::new(150.0, 80.0), // Default size
            child_rects: HashMap::new(),
        }
    }

    /// Draw the node as a styled frame at the given screen position
    /// Returns the response and the screen-space rect
    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        screen_pos: Pos2,
        zoom: f32,
        viewport_rect: Rect,
    ) -> Option<NodeResponse> {
        let _node_id = egui::Id::new(format!(
            "node_{}_{}",
            self.generation,
            self.idx.index()
        ));

        // Build properties dictionary
        let properties: Vec<(&str, String)> = vec![
            ("idx", format!("{}", self.idx.index())),
            ("width", format!("{}", self.data.to_token().width.0)),
            ("parents", format!("{}", self.data.parents().len())),
            ("patterns", format!("{}", self.data.child_patterns().len())),
        ];

        // Get child patterns with indices for display and edge connections
        let patterns = self.child_patterns.patterns_with_indices();

        // Calculate node size based on content - two column layout
        let padding = 6.0 * zoom;
        let title_height = 18.0 * zoom;
        let row_height = 16.0 * zoom;
        let char_width = 7.0 * zoom;
        let key_value_gap = 6.0 * zoom;
        let child_frame_height = 18.0 * zoom;
        let child_frame_padding = 3.0 * zoom;
        let child_spacing = 3.0 * zoom;
        let column_gap = 8.0 * zoom;
        let min_width = 80.0 * zoom;

        // Find max key and value widths for properties column
        let max_key_width = properties
            .iter()
            .map(|(k, _)| k.len() as f32 * char_width)
            .fold(0.0f32, |a, b| a.max(b));
        let max_value_width = properties
            .iter()
            .map(|(_, v)| v.len() as f32 * char_width)
            .fold(0.0f32, |a, b| a.max(b));
        let props_column_width =
            max_key_width + key_value_gap + max_value_width;

        // Calculate patterns column width (using name length from tuple)
        let max_pattern_width = patterns
            .iter()
            .map(|pat| {
                pat.iter()
                    .map(|(name, _idx)| {
                        name.len() as f32 * char_width
                            + child_frame_padding * 2.0
                    })
                    .sum::<f32>()
                    + (pat.len().saturating_sub(1)) as f32 * child_spacing
            })
            .fold(0.0f32, |a, b| a.max(b));
        let patterns_column_width = if patterns.is_empty() {
            0.0
        } else {
            max_pattern_width
        };

        // Total width: both columns + gap + padding
        let title_width = self.name.len() as f32 * 8.0 * zoom;
        let columns_width = if patterns.is_empty() {
            props_column_width
        } else {
            props_column_width + column_gap + patterns_column_width
        };
        let node_width =
            columns_width.max(title_width).max(min_width) + padding * 2.0;

        // Height: taller of the two columns
        let props_height = properties.len() as f32 * row_height;
        let patterns_height =
            patterns.len() as f32 * (child_frame_height + child_spacing);
        let content_height = props_height.max(patterns_height);
        let node_height = title_height + content_height + padding;

        self.cached_size = Vec2::new(node_width / zoom, node_height / zoom);

        let node_rect =
            Rect::from_min_size(screen_pos, Vec2::new(node_width, node_height));

        // Check if node is visible in viewport
        if !viewport_rect.intersects(node_rect) {
            return None;
        }

        // Allocate the rect and get response
        let response = ui.allocate_rect(node_rect, Sense::click_and_drag());

        // Determine colors based on state
        let is_labeled = self.graph.labels.read().unwrap().contains(&self.key);
        let is_hovered = response.hovered();
        let is_dragged = response.dragged();

        let base_color = if is_labeled {
            Color32::from_rgb(20, 80, 40)
        } else {
            Color32::from_rgb(45, 50, 60)
        };

        let fill_color = if is_dragged {
            Color32::from_rgb(
                (base_color.r() as u16 + 30).min(255) as u8,
                (base_color.g() as u16 + 30).min(255) as u8,
                (base_color.b() as u16 + 30).min(255) as u8,
            )
        } else if is_hovered {
            Color32::from_rgb(
                (base_color.r() as u16 + 15).min(255) as u8,
                (base_color.g() as u16 + 15).min(255) as u8,
                (base_color.b() as u16 + 15).min(255) as u8,
            )
        } else {
            base_color
        };

        let border_color = if is_hovered || is_dragged {
            Color32::from_rgb(100, 150, 200)
        } else {
            Color32::from_rgb(70, 80, 90)
        };

        // Draw node background with rounded corners
        let painter = ui.painter().with_clip_rect(viewport_rect);
        let rounding = (8.0 * zoom) as u8;

        // Shadow
        let shadow_offset = Vec2::new(3.0, 3.0) * zoom;
        let shadow_rect = node_rect.translate(shadow_offset);
        painter.rect_filled(
            shadow_rect,
            rounding,
            Color32::from_rgba_unmultiplied(0, 0, 0, 40),
        );

        // Main body
        painter.rect(
            node_rect,
            rounding,
            fill_color,
            Stroke::new(2.0 * zoom, border_color),
            StrokeKind::Inside,
        );

        // Title bar
        let title_rect = Rect::from_min_size(
            node_rect.min,
            Vec2::new(node_width, title_height),
        );
        let title_rounding = CornerRadius {
            nw: rounding,
            ne: rounding,
            sw: 0,
            se: 0,
        };
        painter.rect_filled(
            title_rect,
            title_rounding,
            Color32::from_rgba_unmultiplied(0, 0, 0, 30),
        );

        // Title text - centered
        let title_center_x = node_rect.center().x;
        painter.text(
            Pos2::new(title_center_x, node_rect.min.y + 1.0 * zoom),
            egui::Align2::CENTER_TOP,
            &self.name,
            FontId::proportional(12.0 * zoom),
            Color32::WHITE,
        );

        // Content area - two columns: properties on left, patterns on right
        let content_y = node_rect.min.y + title_height;
        let left_x = node_rect.min.x + padding;
        let patterns_x = left_x + props_column_width + column_gap;

        // Left column: properties
        let mut prop_y = content_y;
        for (i, (key, value)) in properties.iter().enumerate() {
            // Alternate row background (only for properties column)
            if i % 2 == 0 {
                let row_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x, prop_y),
                    Vec2::new(props_column_width + padding, row_height),
                );
                painter.rect_filled(
                    row_rect,
                    0,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 15),
                );
            }

            // Key label (left aligned, dimmed)
            painter.text(
                Pos2::new(left_x, prop_y),
                egui::Align2::LEFT_TOP,
                *key,
                FontId::proportional(10.0 * zoom),
                Color32::from_rgb(140, 140, 150),
            );

            // Value (after key)
            let value_x = left_x + max_key_width + key_value_gap;
            painter.text(
                Pos2::new(value_x, prop_y),
                egui::Align2::LEFT_TOP,
                value,
                FontId::proportional(10.0 * zoom),
                Color32::from_rgb(200, 220, 255),
            );

            prop_y += row_height;
        }

        // Right column: child patterns - collect child rects for edge connections
        self.child_rects.clear();

        if !patterns.is_empty() {
            let mut pattern_y = content_y;

            for pattern in patterns.iter() {
                let mut child_x = patterns_x;

                for (i, (child_name, child_idx)) in pattern.iter().enumerate() {
                    let child_width = child_name.len() as f32 * char_width
                        + child_frame_padding * 2.0;
                    let child_rect = Rect::from_min_size(
                        Pos2::new(child_x, pattern_y),
                        Vec2::new(child_width, child_frame_height),
                    );

                    // Store child rect for edge connections (same child can appear multiple times)
                    self.child_rects
                        .entry(*child_idx)
                        .or_default()
                        .push(child_rect);

                    // Child frame background - alternate colors
                    let frame_color = if i % 2 == 0 {
                        Color32::from_rgb(55, 60, 70)
                    } else {
                        Color32::from_rgb(50, 55, 65)
                    };
                    let frame_border = Color32::from_rgb(80, 90, 100);

                    painter.rect(
                        child_rect,
                        3,
                        frame_color,
                        Stroke::new(1.0 * zoom, frame_border),
                        StrokeKind::Inside,
                    );

                    // Child name text
                    painter.text(
                        child_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        child_name,
                        FontId::proportional(10.0 * zoom),
                        Color32::from_rgb(220, 220, 220),
                    );

                    child_x += child_width + child_spacing;
                }

                pattern_y += child_frame_height + child_spacing;
            }
        }

        // Collect ranges (simplified for now)
        let ranges = IndexMap::new();

        Some(NodeResponse {
            response,
            rect: node_rect,
            ranges,
        })
    }
}
