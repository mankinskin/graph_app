//! Task management (start_read, abort, polling).

use super::App;
use tokio_util::sync::CancellationToken;

impl App {
    pub(crate) fn start_read(&mut self) {
        // Create cancellation token for this operation
        let cancellation_token = CancellationToken::new();
        self.cancellation_token = Some(cancellation_token.clone());

        let ctx = self.read_ctx.clone();
        let algorithm = self.selected_algorithm;
        let task = tokio::spawn(async move {
            let mut ctx = ctx.write().await;
            ctx.run_algorithm(algorithm, cancellation_token).await;
        });
        self.read_task = Some(task);
    }

    pub(crate) fn abort(&mut self) {
        println!("Aborting read operation...");

        // Cancel via the cancellation token first
        if let Some(token) = &self.cancellation_token {
            println!("Cancelling via token...");
            token.cancel();
        }

        // Immediately abort the task - don't wait
        if let Some(handle) = self.read_task.take() {
            println!("Aborting task via handle...");
            handle.abort();
        }

        // Clear the cancellation token
        self.cancellation_token = None;
    }

    pub(crate) fn poll_finished_tasks(&mut self) {
        if self
            .read_task
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false)
        {
            let task = self.read_task.take().unwrap();
            // Clear the cancellation token since task is done
            self.cancellation_token = None;
            tokio::runtime::Handle::current().spawn(task);
        }
    }
}
