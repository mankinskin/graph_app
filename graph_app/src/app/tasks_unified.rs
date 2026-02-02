//! Unified task management using the task abstraction.
//!
//! This module provides platform-agnostic task management for the App.

use super::App;
use crate::task::{TaskHandle, TaskResult};

impl App {
    /// Start running the selected algorithm
    pub(crate) fn start_read(&mut self) {
        // Get the current tab's read context - clone Arc to avoid borrow conflict
        let ctx = match self.current_tab() {
            Some(tab) => tab.read_ctx.clone(),
            None => return,
        };

        let algorithm = self.selected_algorithm;
        let output = self.output.clone();
        let _vis = self.current_tab().map(|t| t.vis.clone());

        output.info(format!("Starting {} algorithm...", algorithm));

        // Native: use spawn with blocking
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::task::spawn;
            
            let task = spawn(move |cancellation| {
                println!("Task starting: algorithm = {:?}", algorithm);

                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    let mut ctx_guard = ctx.write().await;
                    ctx_guard
                        .run_algorithm(algorithm, cancellation.token())
                        .await;
                });
            });

            self.current_task = Some(task);
        }

        // Wasm: use spawn_async with truly async execution
        #[cfg(target_arch = "wasm32")]
        {
            use crate::task::spawn_async;
            
            let task = spawn_async(move |cancellation| async move {
                web_sys::console::log_1(
                    &format!("Task starting: algorithm = {:?}", algorithm).into(),
                );

                // Run the async algorithm
                let mut ctx_guard = ctx.write().unwrap();
                ctx_guard.run_algorithm_async(algorithm, cancellation).await;
                
                web_sys::console::log_1(&"Task completed".into());
            });

            self.current_task = Some(task);
        }
    }

    /// Start a test task that waits for 10 seconds asynchronously
    /// This is used to verify that async tasks work correctly on wasm
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn start_test_async_task(&mut self) {
        use crate::task::spawn_async;
        
        let output = self.output.clone();
        output.info("Starting 10-second async test task...");

        let task = spawn_async(move |cancellation| async move {
            for i in 0..10 {
                if cancellation.is_cancelled() {
                    web_sys::console::log_1(&"Test task cancelled".into());
                    return;
                }
                
                web_sys::console::log_1(&format!("Test task: {} seconds elapsed", i + 1).into());
                gloo_timers::future::TimeoutFuture::new(1000).await;
            }
            web_sys::console::log_1(&"Test task completed!".into());
        });

        self.current_task = Some(task);
    }

    /// Abort the currently running task
    pub(crate) fn abort(&mut self) {
        self.output.warn("Aborting operation...");

        if let Some(task) = &self.current_task {
            task.abort();
        }

        self.current_task = None;
        self.output.warn("Operation aborted.");
    }

    /// Poll for finished tasks and handle their results
    pub(crate) fn poll_finished_tasks(&mut self) {
        let task_finished = self
            .current_task
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false);

        if task_finished {
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

    /// Check if a task is currently running
    pub(crate) fn is_task_running(&self) -> bool {
        self.current_task
            .as_ref()
            .map(|t| t.is_running())
            .unwrap_or(false)
    }

    /// Handle the result of a completed task
    #[allow(unused)]
    fn handle_task_result(
        &mut self,
        result: TaskResult,
    ) {
        match result {
            TaskResult::Success => {
                self.output.success("Algorithm completed successfully.");
            },
            TaskResult::Cancelled => {
                self.output.warn("Algorithm was cancelled.");
            },
            TaskResult::Panicked(msg) => {
                self.output.error(format!("Algorithm panicked: {}", msg));
            },
        }
    }
}
