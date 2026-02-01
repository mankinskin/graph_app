//! Task management for WebAssembly (start_read, abort).
//! Uses wasm-bindgen-futures for async execution.

use super::App;
use crate::read::ReadCtx;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

impl App {
    pub(crate) fn start_read(&mut self) {
        // Get the current tab's read context - clone Arc to avoid borrow conflict
        let ctx = match self.current_tab() {
            Some(tab) => tab.read_ctx.clone(),
            None => return,
        };

        // Create cancellation flag for this operation
        let cancelled = Arc::new(AtomicBool::new(false));
        self.cancelled = Some(cancelled.clone());

        let algorithm = self.selected_algorithm;
        let output = self.output.clone();
        let vis = self.current_tab().map(|t| t.vis.clone());
        
        output.info(format!("Starting {} algorithm...", algorithm));
        
        // Set running flag
        self.is_running = true;
        self.running_flag.store(true, Ordering::SeqCst);
        
        // Clone what we need for the async block
        let running_flag = self.running_flag.clone();
        
        wasm_bindgen_futures::spawn_local(async move {
            {
                let mut ctx_guard = ctx.write().unwrap();
                ctx_guard.run_algorithm_sync(algorithm, &cancelled);
            }
            
            // Mark visualization as dirty
            if let Some(vis_arc) = vis {
                if let Ok(mut vis) = vis_arc.write() {
                    vis.mark_dirty();
                }
            }
            
            output.success(format!("{} algorithm completed.", algorithm));
            running_flag.store(false, Ordering::SeqCst);
        });
    }

    pub(crate) fn abort(&mut self) {
        self.output.warn("Aborting read operation...");

        // Cancel via the cancellation flag
        if let Some(cancelled) = &self.cancelled {
            self.output.info("Cancelling...");
            cancelled.store(true, Ordering::SeqCst);
        }

        // Clear the cancellation flag
        self.cancelled = None;
        self.is_running = false;
        
        self.output.warn("Read operation aborted.");
    }

    pub(crate) fn poll_finished_tasks(&mut self) {
        // Check if the async task has finished
        if self.is_running && !self.running_flag.load(Ordering::SeqCst) {
            self.is_running = false;
            self.cancelled = None;
            
            // Mark the current tab's visualization as dirty so it rebuilds
            if let Some(tab) = self.current_tab() {
                if let Some(mut vis) = tab.vis_mut() {
                    vis.mark_dirty();
                }
            }
        }
    }
}
