//! Central panel with closeable graph tabs and inserter window.

use eframe::egui::{
    self,
    Ui,
};

use super::{
    App,
    GraphTab,
};
use crate::{
    examples::{
        build_graph1,
        build_graph2,
        build_graph3,
    },
    widgets::{EditableLabel, Inserter},
};

impl App {
    pub(crate) fn central_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                // Tab bar with close buttons and new tab button
                self.show_tab_bar(ui);
                ui.separator();

                // Tab-specific menu bar
                self.show_tab_menu_bar(ui);
                ui.separator();

                // Get viewport rect for constraining windows
                let viewport_rect = ui.available_rect_before_wrap();

                // Show the graph for the selected tab and handle clicks
                let mut clicked_node = None;
                let mut background_clicked = false;
                if let Some(mut vis) = self.vis_mut() {
                    let response = vis.show(ui);
                    clicked_node = response.clicked_node;
                    background_clicked = response.background_clicked;
                }

                // Update selection if a node was clicked, clear if background was clicked
                if let Some(key) = clicked_node {
                    if let Some(tab) = self.current_tab_mut() {
                        tab.selected_node = Some(key);
                    }
                } else if background_clicked {
                    if let Some(tab) = self.current_tab_mut() {
                        tab.selected_node = None;
                    }
                }

                // Show inserter as a floating window within the central panel
                self.show_inserter_window(ctx, viewport_rect);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }

    fn show_tab_bar(
        &mut self,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            let mut tab_to_close: Option<usize> = None;
            let mut tab_to_select: Option<usize> = None;
            let mut tab_renamed: Option<(usize, String)> = None;
            let tab_count = self.tabs.len();

            for tab in &mut self.tabs {
                let is_selected = tab.id == self.selected_tab_id;
                let is_editing = tab.label_state.editing;

                ui.horizontal(|ui| {
                    let response =
                        EditableLabel::new(&tab.name, &mut tab.label_state)
                            .selected(is_selected)
                            .prefix("📊")
                            .show(ui);

                    if response.clicked {
                        tab_to_select = Some(tab.id);
                    }

                    if let Some(new_name) = response.renamed {
                        tab_renamed = Some((tab.id, new_name));
                    }

                    // Close button (only show if more than one tab and not editing)
                    if tab_count > 1 && !is_editing {
                        if ui
                            .small_button("x")
                            .on_hover_text("Close tab")
                            .clicked()
                        {
                            tab_to_close = Some(tab.id);
                        }
                    }
                });

                ui.separator();
            }

            // New tab button
            if ui.button("+").on_hover_text("New tab").clicked() {
                self.create_new_tab();
            }

            // Handle tab selection
            if let Some(id) = tab_to_select {
                self.selected_tab_id = id;
            }

            // Handle tab close
            if let Some(id) = tab_to_close {
                self.close_tab(id);
            }

            // Handle rename
            if let Some((id, new_name)) = tab_renamed {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                    tab.name = new_name;
                }
            }
        });
    }

    fn show_tab_menu_bar(
        &mut self,
        ui: &mut Ui,
    ) {
        egui::MenuBar::new().ui(ui, |ui| {
            // Edit menu
            ui.menu_button("Edit", |ui| {
                // Presets submenu
                ui.menu_button("Load Preset", |ui| {
                    if let Some(ctx) = self.ctx() {
                        if ui.button("Graph 1").clicked() {
                            ctx.graph().set_graph(build_graph1());
                            if let Some(mut vis) = self.vis_mut() {
                                vis.mark_dirty();
                            }
                            ui.close();
                        }
                        if ui.button("Graph 2").clicked() {
                            ctx.graph().set_graph(build_graph2());
                            if let Some(mut vis) = self.vis_mut() {
                                vis.mark_dirty();
                            }
                            ui.close();
                        }
                        if ui.button("Graph 3").clicked() {
                            ctx.graph().set_graph(build_graph3());
                            if let Some(mut vis) = self.vis_mut() {
                                vis.mark_dirty();
                            }
                            ui.close();
                        }
                    }
                });

                ui.separator();

                if ui.button("Clear").clicked() {
                    if let Some(mut ctx) = self.ctx_mut() {
                        ctx.graph_mut().clear();
                    }
                    if let Some(mut vis) = self.vis_mut() {
                        vis.mark_dirty();
                    }
                    ui.close();
                }
            });
        });
    }

    pub(crate) fn create_new_tab(&mut self) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let name = format!("Graph {}", id + 1);
        self.tabs.push(GraphTab::new(id, name));
        self.selected_tab_id = id;
    }

    pub(crate) fn close_tab(
        &mut self,
        id: usize,
    ) {
        if self.tabs.len() <= 1 {
            return; // Don't close the last tab
        }

        // Find the tab index
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(idx);

            // If we closed the selected tab, select another one
            if self.selected_tab_id == id {
                // Select the previous tab, or the first one if we closed the first
                let new_idx = idx.saturating_sub(1).min(self.tabs.len() - 1);
                self.selected_tab_id = self.tabs[new_idx].id;
            }
        }
    }

    fn show_inserter_window(
        &mut self,
        ctx: &egui::Context,
        viewport_rect: egui::Rect,
    ) {
        if !self.inserter_open {
            return;
        }

        let mut inserter_open = self.inserter_open;
        let window_width = 350.0;
        let right_aligned_pos = egui::pos2(
            viewport_rect.right() - window_width - 10.0,
            viewport_rect.top() + 10.0,
        );

        let mut window = egui::Window::new("✏ Inserter")
            .open(&mut inserter_open)
            .resizable(true)
            .default_width(window_width)
            .constrain_to(viewport_rect);

        // Keep right-aligned unless manually moved
        if !self.inserter_manually_moved {
            window = window.current_pos(right_aligned_pos);
        } else {
            window = window.default_pos(right_aligned_pos);
        }

        let response = window.show(ctx, |ui| {
            self.show_inserter_content(ui);
        });

        // Detect if window was dragged
        if let Some(inner) = response {
            if inner.response.dragged() {
                self.inserter_manually_moved = true;
            }
        }

        self.inserter_open = inserter_open;
    }

    fn show_inserter_content(
        &mut self,
        ui: &mut Ui,
    ) {
        let is_running = self.is_task_running();

        // Get texts from the read context
        let mut texts = if let Some(mut read_ctx) = self.ctx_mut() {
            std::mem::take(&mut read_ctx.graph_mut().insert_texts)
        } else {
            return;
        };

        let response = Inserter::new(
            &mut self.selected_algorithm,
            &mut texts,
            is_running,
        )
        .show(ui);

        // Put texts back
        if let Some(mut read_ctx) = self.ctx_mut() {
            read_ctx.graph_mut().insert_texts = texts;
        }

        // Handle response
        if response.run_clicked {
            self.start_read();
        }

        if response.cancel_clicked {
            self.abort();
        }

        if response.clear_clicked {
            if let Some(mut ctx) = self.ctx_mut() {
                ctx.graph_mut().clear();
            }
            if let Some(mut vis) = self.vis_mut() {
                vis.mark_dirty();
            }
        }

        #[cfg(target_arch = "wasm32")]
        if response.test_async_clicked {
            self.start_test_async_task();
        }
    }
}
