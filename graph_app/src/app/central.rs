//! Central panel with closeable graph tabs and inserter window.

use eframe::egui::{self, Ui};
use strum::IntoEnumIterator;

use super::{App, GraphTab};
use crate::algorithm::Algorithm;

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

                // Get viewport rect for constraining windows
                let viewport_rect = ui.available_rect_before_wrap();

                // Show the graph for the selected tab
                if let Some(mut vis) = self.vis_mut() {
                    vis.show(ui)
                }

                // Show inserter as a floating window within the central panel
                self.show_inserter_window(ctx, viewport_rect);

                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut tab_to_close: Option<usize> = None;
            let mut tab_to_select: Option<usize> = None;

            for tab in &self.tabs {
                let is_selected = tab.id == self.selected_tab_id;
                
                ui.horizontal(|ui| {
                    // Tab button
                    if ui
                        .selectable_label(is_selected, format!("📊 {}", &tab.name))
                        .clicked()
                    {
                        tab_to_select = Some(tab.id);
                    }

                    // Close button (only show if more than one tab)
                    if self.tabs.len() > 1 {
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
        });
    }

    pub(crate) fn create_new_tab(&mut self) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let name = format!("Graph {}", id + 1);
        self.tabs.push(GraphTab::new(id, name));
        self.selected_tab_id = id;
    }

    pub(crate) fn close_tab(&mut self, id: usize) {
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
        egui::Window::new("✏ Inserter")
            .open(&mut inserter_open)
            .resizable(true)
            .default_width(350.0)
            .constrain_to(viewport_rect)
            .show(ctx, |ui| {
                self.show_inserter_content(ui);
            });
        self.inserter_open = inserter_open;
    }

    fn show_inserter_content(&mut self, ui: &mut Ui) {
        // Show currently selected algorithm
        ui.horizontal(|ui| {
            ui.label("Algorithm:");
            egui::ComboBox::from_id_salt("inserter_algorithm")
                .selected_text(self.selected_algorithm.to_string())
                .show_ui(ui, |ui| {
                    for algorithm in Algorithm::iter() {
                        ui.selectable_value(
                            &mut self.selected_algorithm,
                            algorithm,
                            algorithm.to_string(),
                        );
                    }
                });
        });

        ui.add_space(5.0);
        ui.label(self.selected_algorithm.description());

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        ui.label("Input texts:");
        ui.add_space(5.0);

        if let Some(mut read_ctx) = self.ctx_mut() {
            let texts = &mut read_ctx.graph_mut().insert_texts;
            let mut to_remove = None;

            for (i, text) in texts.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(text).desired_width(280.0));
                    if ui.button("✖").on_hover_text("Remove").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(idx) = to_remove {
                texts.remove(idx);
            }

            ui.add_space(5.0);
            if ui.button("+ Add Text").clicked() {
                texts.push(String::new());
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            let is_running = self.read_task.is_some();

            if ui
                .add_enabled(!is_running, egui::Button::new("▶ Run"))
                .clicked()
            {
                self.start_read();
            }

            if is_running {
                if ui.button("⏹ Cancel").clicked() {
                    self.abort();
                }
                ui.spinner();
                ui.label("Processing...");
            }
        });
    }
}
