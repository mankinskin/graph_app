//! Native-specific task implementations.

use std::sync::Arc;

use async_std::sync::RwLock;

use crate::{
    algorithm::Algorithm,
    read::ReadCtx,
    task::CancellationHandle,
};

/// Run an algorithm task on the native platform.
pub(crate) async fn run_algorithm_task(
    ctx: Arc<RwLock<ReadCtx>>,
    algorithm: Algorithm,
    cancellation: CancellationHandle,
) {
    println!("Task starting: algorithm = {:?}", algorithm);
    let mut ctx_guard = ctx.write().await;
    ctx_guard
        .run_algorithm(algorithm, cancellation.token())
        .await;
    println!("Task completed");
}
