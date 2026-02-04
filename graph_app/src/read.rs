//! Reading context for algorithm execution.
//!
//! This module provides the `ReadCtx` which manages the graph state and
//! executes algorithms with proper cancellation support.

use ngrams::graph::StatusHandle;
#[cfg(feature = "persistence")]
use serde::*;

use crate::{
    algorithm::Algorithm,
    graph::*,
    task::CancellationHandle,
};

use context_trace::{
    graph::HypergraphRef,
    Token,
};
use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

// ============================================================================
// ReadCtx - Core struct
// ============================================================================

/// Context for reading and processing graph data.
#[derive(Debug)]
pub struct ReadCtx {
    graph: Graph,
    status: Option<StatusHandle>,
}

impl ReadCtx {
    /// Create a new read context with the given graph.
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            status: None,
        }
    }

    /// Get the current status handle, if any.
    pub fn status(&self) -> Option<&StatusHandle> {
        self.status.as_ref()
    }

    /// Get a reference to the graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get a mutable reference to the graph.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }
}

// ============================================================================
// Algorithm execution - unified interface
// ============================================================================

impl ReadCtx {
    /// Execute the selected algorithm on the input texts.
    ///
    /// This is the unified entry point for algorithm execution on both platforms.
    pub async fn run_algorithm(
        &mut self,
        algorithm: Algorithm,
        cancellation: CancellationHandle,
    ) {
        log_info(&format!("Running algorithm: {:?}", algorithm));

        match algorithm {
            Algorithm::NgramsParseCorpus => {
                self.run_ngrams_parse_corpus(&cancellation).await;
            },
            Algorithm::ContextRead => {
                self.run_context_read(&cancellation).await;
            },
            Algorithm::ContextInsert => {
                self.run_context_insert(&cancellation).await;
            },
        }

        log_info("Task done.");
    }

    /// Run ngrams::parse_corpus algorithm.
    async fn run_ngrams_parse_corpus(
        &mut self,
        cancellation: &CancellationHandle,
    ) {
        use ngrams::graph::{
            traversal::pass::CancelReason,
            Corpus,
            Status,
            StatusHandle,
        };

        log_info("Starting ngrams parse corpus...");

        let graph = self.graph.graph.clone();
        let labels = self.graph.labels.clone();
        let insert_texts = self.graph.insert_texts.clone();

        // Guard against empty corpus
        let non_empty_texts: Vec<_> = insert_texts
            .iter()
            .filter(|t| !t.is_empty())
            .cloned()
            .collect();

        if non_empty_texts.is_empty() {
            log_info("No text to parse (insert_texts is empty)");
            return;
        }

        log_info(&format!("Insert texts: {:?}", non_empty_texts));

        let status = StatusHandle::from(Status::new(non_empty_texts.clone()));
        self.status = Some(status.clone());

        let corpus_name = {
            let mut hasher = DefaultHasher::new();
            non_empty_texts.hash(&mut hasher);
            format!("{:?}", hasher.finish())
        };
        let corpus = Corpus::new(corpus_name.clone(), non_empty_texts);

        log_info(&format!("Created corpus: {}", corpus_name));

        // Execute the parse - platform-specific
        let result = self
            .execute_parse_corpus(corpus, status, cancellation)
            .await;

        match result {
            Ok(res) => {
                self.graph.insert_texts.clear();
                *graph.write().unwrap() = res.graph.into();
                *labels.write().unwrap() = res.labels;
                log_info("Ngrams parse corpus completed successfully");
            },
            Err(CancelReason::Cancelled) => {
                log_info("Parse operation was cancelled");
            },
            Err(CancelReason::Error) => {
                log_error("Parse operation encountered an error");
            },
            Err(CancelReason::EmptyVocabulary) => {
                log_info("Parse operation cancelled: empty vocabulary");
            },
        }
    }

    /// Run context-read algorithm.
    async fn run_context_read(
        &mut self,
        cancellation: &CancellationHandle,
    ) {
        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        let combined_text: String = insert_texts.join("");

        if combined_text.is_empty() {
            log_info("No text to read");
            return;
        }

        let mut read_ctx = context_read::context::ReadCtx::new(
            graph_ref.clone(),
            combined_text.chars(),
        );

        // Yield before starting (wasm only)
        yield_if_wasm().await;

        // Process in chunks to allow UI to remain responsive
        let mut iteration = 0;
        while !cancellation.is_cancelled() {
            if read_ctx.next().is_none() {
                break;
            }

            iteration += 1;
            // Yield periodically on wasm to keep UI responsive
            if iteration % 100 == 0 {
                yield_if_wasm().await;
            }
        }

        if cancellation.is_cancelled() {
            log_info("Context read operation was cancelled");
        } else {
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            log_info("Context read completed successfully");
        }
    }

    /// Run context-insert algorithm.
    async fn run_context_insert(
        &mut self,
        cancellation: &CancellationHandle,
    ) {
        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        let mut insert_ctx =
            context_insert::InsertCtx::<Token>::from(graph_ref.clone());

        // Yield before starting (wasm only)
        yield_if_wasm().await;

        for (i, text) in insert_texts.iter().enumerate() {
            if cancellation.is_cancelled() {
                log_info("Context insert operation was cancelled");
                return;
            }

            if text.is_empty() {
                continue;
            }

            log_info(&format!("Inserting text {}: {}", i + 1, text));

            // Get tokens for insertion
            let tokens = self.get_tokens_for_text(&graph_ref, text);

            match insert_ctx.insert(tokens) {
                Ok(_result) => {
                    log_info(&format!("Inserted: {}", text));
                },
                Err(err) => {
                    log_error(&format!(
                        "Error inserting '{}': {:?}",
                        text, err
                    ));
                },
            }

            // Yield after each insert (wasm only)
            yield_if_wasm().await;
        }

        if !cancellation.is_cancelled() {
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            log_info("Context insert completed successfully");
        }
    }
}

// ============================================================================
// Platform-specific implementations
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl ReadCtx {
    /// Execute parse_corpus using tokio spawn_blocking.
    async fn execute_parse_corpus(
        &self,
        corpus: ngrams::graph::Corpus,
        status: StatusHandle,
        cancellation: &CancellationHandle,
    ) -> Result<
        ngrams::graph::ParseResult,
        ngrams::graph::traversal::pass::CancelReason,
    > {
        use ngrams::cancellation::Cancellation;

        let ngrams_cancellation = Cancellation::from(cancellation.token());

        // Run in blocking task pool
        let result = tokio::task::spawn_blocking(move || {
            ngrams::graph::parse_corpus(corpus, status, ngrams_cancellation)
        })
        .await;

        match result {
            Ok(res) => res,
            Err(join_error) => {
                log_error(&format!("Parse task panicked: {:?}", join_error));
                Err(ngrams::graph::traversal::pass::CancelReason::Error)
            },
        }
    }

    /// Get tokens for text insertion.
    fn get_tokens_for_text(
        &self,
        graph_ref: &HypergraphRef,
        text: &str,
    ) -> Vec<Token> {
        graph_ref.expect_atom_children(text.chars()).to_vec()
    }
}

#[cfg(target_arch = "wasm32")]
impl ReadCtx {
    /// Execute parse_corpus synchronously (wasm runs on main thread).
    async fn execute_parse_corpus(
        &self,
        corpus: ngrams::graph::Corpus,
        status: StatusHandle,
        cancellation: &CancellationHandle,
    ) -> Result<
        ngrams::graph::ParseResult,
        ngrams::graph::traversal::pass::CancelReason,
    > {
        use ngrams::cancellation::Cancellation;

        let ngrams_cancellation = Cancellation::from(cancellation.flag());

        // Yield to allow UI to update before starting heavy work
        yield_if_wasm().await;

        // Run synchronously (wasm is single-threaded)
        ngrams::graph::parse_corpus(corpus, status, ngrams_cancellation)
    }

    /// Get tokens for text insertion.
    fn get_tokens_for_text(
        &self,
        graph_ref: &HypergraphRef,
        text: &str,
    ) -> Vec<Token> {
        use context_trace::graph::vertex::has_vertex_index::HasVertexIndex;

        let atom_indices = graph_ref.new_atom_indices(text.chars());
        atom_indices
            .into_iter()
            .map(|idx| Token::new(idx.vertex_index(), 1))
            .collect()
    }
}

// ============================================================================
// Cross-platform utilities
// ============================================================================

/// Log an info message.
#[cfg(not(target_arch = "wasm32"))]
fn log_info(msg: &str) {
    println!("{}", msg);
}

/// Log an info message.
#[cfg(target_arch = "wasm32")]
fn log_info(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

/// Log an error message.
#[cfg(not(target_arch = "wasm32"))]
fn log_error(msg: &str) {
    eprintln!("{}", msg);
}

/// Log an error message.
#[cfg(target_arch = "wasm32")]
fn log_error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}

/// Yield to the event loop on wasm, no-op on native.
#[cfg(not(target_arch = "wasm32"))]
async fn yield_if_wasm() {
    // No-op on native
}

/// Yield to the event loop on wasm.
#[cfg(target_arch = "wasm32")]
async fn yield_if_wasm() {
    gloo_timers::future::TimeoutFuture::new(0).await;
}
