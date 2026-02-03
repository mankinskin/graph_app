//! Wasm-specific task implementations.

use std::sync::{Arc, RwLock};

use crate::{
    algorithm::Algorithm,
    read::ReadCtx,
    task::CancellationHandle,
};

/// Run an algorithm task on the wasm platform.
pub(crate) async fn run_algorithm_task(
    ctx: Arc<RwLock<ReadCtx>>,
    algorithm: Algorithm,
    cancellation: CancellationHandle,
) {
    web_sys::console::log_1(&format!("Task starting: algorithm = {:?}", algorithm).into());

    {
        let mut ctx_guard = ctx.write().unwrap();
        ctx_guard.run_algorithm_async(algorithm, cancellation).await;
    }

    web_sys::console::log_1(&"Task completed".into());
}
