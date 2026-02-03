//! Application task management - integration between App and task execution.
//!
//! This module provides the glue between the App state and the task execution system.
//! It handles spawning algorithm tasks, polling for completion, and updating UI state.

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

use super::App;
use crate::task::{CancellationHandle, TaskHandle, TaskResult};

impl App {
    /// Start a test async task (10 seconds).
    ///
    /// This is used to verify that async tasks work correctly on both platforms.
    #[allow(dead_code)]
    pub(crate) fn start_test_async_task(&mut self) {
        let output = self.output.clone();
        output.info("Starting 10-second async test task...");

        #[cfg(not(target_arch = "wasm32"))]
        let task = TaskHandle::spawn(move |cancellation| async move {
            run_test_async_task_native(cancellation).await;
        });

        #[cfg(target_arch = "wasm32")]
        let task = TaskHandle::spawn(move |cancellation| async move {
            run_test_async_task_wasm(cancellation).await;
        });

        self.current_task = Some(task);
    }

    /// Start a test blocking task.
    ///
    /// This tests the blocking task execution:
    /// - Native: Uses tokio's spawn_blocking
    /// - Wasm: Uses Web Workers (or fallback)
    #[allow(dead_code)]
    pub(crate) fn start_test_blocking_task(&mut self) {
        let output = self.output.clone();
        output.info("Starting blocking test task...");

        let task = TaskHandle::spawn_blocking(move |cancellation| {
            for i in 0..10 {
                if cancellation.is_cancelled() {
                    return;
                }
                // Simulate CPU-intensive work
                std::thread::sleep(std::time::Duration::from_millis(500));
                #[cfg(not(target_arch = "wasm32"))]
                println!("Blocking task: {} iterations completed", i + 1);
            }
        });

        self.current_task = Some(task);
    }

    /// Start running the selected algorithm.
    pub(crate) fn start_read(&mut self) {
        let algorithm = self.selected_algorithm;
        let output = self.output.clone();

        output.info(format!("Starting {} algorithm...", algorithm));

        let ctx = match self.current_tab() {
            Some(tab) => tab.read_ctx.clone(),
            None => return,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let task = TaskHandle::spawn(move |cancellation| async move {
            native::run_algorithm_task(ctx, algorithm, cancellation).await;
        });

        #[cfg(target_arch = "wasm32")]
        let task = TaskHandle::spawn(move |cancellation| async move {
            wasm::run_algorithm_task(ctx, algorithm, cancellation).await;
        });

        self.current_task = Some(task);
    }

    /// Abort the currently running task.
    pub(crate) fn abort(&mut self) {
        self.output.warn("Aborting operation...");

        if let Some(task) = &self.current_task {
            task.abort();
        }

        self.current_task = None;
        self.output.warn("Operation aborted.");
    }

    /// Poll for finished tasks and handle their results.
    pub(crate) fn poll_finished_tasks(&mut self) {
        let task_finished = self
            .current_task
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false);

        if task_finished {
            #[allow(unused_variables)]
            let task = self.current_task.take().unwrap();

            // Handle the result
            #[cfg(target_arch = "wasm32")]
            if let Some(result) = task.try_get_result() {
                self.handle_task_result(result);
            }

            // Mark the current tab's visualization as dirty
            if let Some(tab) = self.current_tab() {
                if let Some(mut vis) = tab.vis_mut() {
                    vis.mark_dirty();
                }
            }
        }
    }

    /// Check if a task is currently running.
    pub(crate) fn is_task_running(&self) -> bool {
        self.current_task
            .as_ref()
            .map(|t| t.is_running())
            .unwrap_or(false)
    }

    /// Handle the result of a completed task.
    #[allow(dead_code)]
    fn handle_task_result(
        &mut self,
        result: TaskResult,
    ) {
        match result {
            TaskResult::Success => {
                self.output.success("Algorithm completed successfully.");
            }
            TaskResult::Cancelled => {
                self.output.warn("Algorithm was cancelled.");
            }
            TaskResult::Panicked(msg) => {
                self.output.error(format!("Algorithm panicked: {}", msg));
            }
        }
    }
}

// ============================================================================
// Platform-specific test task implementations
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
async fn run_test_async_task_native(cancellation: CancellationHandle) {
    for i in 0..10 {
        if cancellation.is_cancelled() {
            println!("Test task cancelled");
            return;
        }
        println!("Test task: {} seconds elapsed", i + 1);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    println!("Test task completed!");
}

#[cfg(target_arch = "wasm32")]
async fn run_test_async_task_wasm(cancellation: CancellationHandle) {
    for i in 0..10 {
        if cancellation.is_cancelled() {
            web_sys::console::log_1(&"Test task cancelled".into());
            return;
        }
        web_sys::console::log_1(&format!("Test task: {} seconds elapsed", i + 1).into());
        gloo_timers::future::TimeoutFuture::new(1000).await;
    }
    web_sys::console::log_1(&"Test task completed!".into());
}
