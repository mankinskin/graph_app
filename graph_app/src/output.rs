//! Output buffer for capturing and displaying program output.

use std::sync::{Arc, Mutex};

/// A line of output with optional styling
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub text: String,
    pub level: OutputLevel,
}

/// Level/category for output lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl OutputLevel {
    pub fn color(&self) -> eframe::egui::Color32 {
        use eframe::egui::Color32;
        match self {
            OutputLevel::Info => Color32::GRAY,
            OutputLevel::Success => Color32::GREEN,
            OutputLevel::Warning => Color32::YELLOW,
            OutputLevel::Error => Color32::from_rgb(255, 100, 100),
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            OutputLevel::Info => "[INFO]",
            OutputLevel::Success => "[OK]",
            OutputLevel::Warning => "[WARN]",
            OutputLevel::Error => "[ERROR]",
        }
    }
}

/// Thread-safe output buffer
#[derive(Debug, Clone, Default)]
pub struct OutputBuffer {
    lines: Arc<Mutex<Vec<OutputLine>>>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a line with the given level
    pub fn push(&self, text: impl Into<String>, level: OutputLevel) {
        if let Ok(mut lines) = self.lines.lock() {
            lines.push(OutputLine {
                text: text.into(),
                level,
            });
        }
    }

    /// Add an info line
    pub fn info(&self, text: impl Into<String>) {
        self.push(text, OutputLevel::Info);
    }

    /// Add a success line
    pub fn success(&self, text: impl Into<String>) {
        self.push(text, OutputLevel::Success);
    }

    /// Add a warning line
    pub fn warn(&self, text: impl Into<String>) {
        self.push(text, OutputLevel::Warning);
    }

    /// Add an error line
    pub fn error(&self, text: impl Into<String>) {
        self.push(text, OutputLevel::Error);
    }

    /// Get all lines
    pub fn lines(&self) -> Vec<OutputLine> {
        self.lines.lock().map(|l| l.clone()).unwrap_or_default()
    }

    /// Clear all output
    pub fn clear(&self) {
        if let Ok(mut lines) = self.lines.lock() {
            lines.clear();
        }
    }

    /// Get the number of lines
    pub fn len(&self) -> usize {
        self.lines.lock().map(|l| l.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
