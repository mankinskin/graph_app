//! A label that can be edited by double-clicking.

use eframe::egui::{
    self,
    Response,
    Ui,
    Widget,
};

/// Response from an editable label interaction.
pub struct EditableLabelResponse {
    /// The label was clicked (single click).
    pub clicked: bool,
    /// The label was renamed (new name returned).
    pub renamed: Option<String>,
}

/// State for an editable label, stored externally.
#[derive(Debug, Default, Clone)]
pub struct EditableLabelState {
    pub editing: bool,
    pub buffer: String,
}

impl EditableLabelState {
    pub fn start_editing(
        &mut self,
        current_text: &str,
    ) {
        self.editing = true;
        self.buffer = current_text.to_string();
    }

    pub fn cancel(&mut self) {
        self.editing = false;
        self.buffer.clear();
    }

    pub fn finish(&mut self) -> Option<String> {
        if self.editing {
            self.editing = false;
            let result = if self.buffer.trim().is_empty() {
                None
            } else {
                Some(std::mem::take(&mut self.buffer))
            };
            self.buffer.clear();
            result
        } else {
            None
        }
    }
}

/// A label widget that shows a text edit when double-clicked.
pub struct EditableLabel<'a> {
    text: &'a str,
    state: &'a mut EditableLabelState,
    selected: bool,
    prefix: Option<&'a str>,
}

impl<'a> EditableLabel<'a> {
    pub fn new(
        text: &'a str,
        state: &'a mut EditableLabelState,
    ) -> Self {
        Self {
            text,
            state,
            selected: false,
            prefix: None,
        }
    }

    /// Mark this label as selected (highlighted).
    pub fn selected(
        mut self,
        selected: bool,
    ) -> Self {
        self.selected = selected;
        self
    }

    /// Add a prefix (e.g., an icon) before the text.
    pub fn prefix(
        mut self,
        prefix: &'a str,
    ) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
    ) -> EditableLabelResponse {
        let mut clicked = false;
        let mut renamed = None;

        if self.state.editing {
            let response = ui.text_edit_singleline(&mut self.state.buffer);

            // Auto-focus on first frame
            if !response.has_focus() {
                response.request_focus();
            }

            // Handle completion
            if response.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.state.cancel();
                } else {
                    renamed = self.state.finish();
                }
            }
        } else {
            let label_text = match self.prefix {
                Some(prefix) => format!("{} {}", prefix, self.text),
                None => self.text.to_string(),
            };

            let response = ui.selectable_label(self.selected, label_text);

            if response.clicked() {
                clicked = true;
            }

            if response.double_clicked() {
                self.state.start_editing(self.text);
            }
        }

        EditableLabelResponse { clicked, renamed }
    }
}

impl<'a> Widget for EditableLabel<'a> {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        // For Widget trait, we just show and return a dummy response
        // Use show() method for full functionality
        let _ = self.show(ui);
        ui.label("") // Placeholder response
    }
}
