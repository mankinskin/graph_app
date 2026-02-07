//! Task execution results.

/// Result of a task execution.
#[derive(Debug, Clone)]
pub(crate) enum TaskResult {
    /// Task completed successfully.
    Success,
    /// Task was cancelled before completion.
    Cancelled,
    /// Task panicked with an error message.
    Panicked(String),
}

impl TaskResult {
    /// Returns `true` if the task completed successfully.
    pub(crate) fn is_success(&self) -> bool {
        matches!(self, TaskResult::Success)
    }

    /// Returns `true` if the task was cancelled.
    pub(crate) fn is_cancelled(&self) -> bool {
        matches!(self, TaskResult::Cancelled)
    }

    /// Returns `true` if the task panicked.
    pub(crate) fn is_panicked(&self) -> bool {
        matches!(self, TaskResult::Panicked(_))
    }

    /// Get the panic message if the task panicked.
    pub(crate) fn panic_message(&self) -> Option<&str> {
        match self {
            TaskResult::Panicked(msg) => Some(msg),
            _ => None,
        }
    }
}

impl std::fmt::Display for TaskResult {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            TaskResult::Success => write!(f, "Success"),
            TaskResult::Cancelled => write!(f, "Cancelled"),
            TaskResult::Panicked(msg) => write!(f, "Panicked: {}", msg),
        }
    }
}
