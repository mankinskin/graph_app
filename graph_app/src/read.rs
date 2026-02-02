#[cfg(target_arch = "wasm32")]
use ngrams::graph::StatusHandle;
#[cfg(not(target_arch = "wasm32"))]
use ngrams::graph::{
    traversal::pass::CancelReason,
    Status,
    StatusHandle,
};
#[cfg(feature = "persistence")]
use serde::*;

#[allow(unused)]
use crate::algorithm::Algorithm;
#[allow(unused)]
use crate::graph::*;

#[cfg(target_arch = "wasm32")]
use context_trace::graph::vertex::has_vertex_index::HasVertexIndex;
use context_trace::{
    graph::HypergraphRef,
    Token,
};
#[cfg(not(target_arch = "wasm32"))]
use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ReadCtx {
    graph: Graph,
    status: Option<ngrams::graph::StatusHandle>,
}
impl ReadCtx {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            status: None,
        }
    }
    pub fn status(&self) -> Option<&StatusHandle> {
        self.status.as_ref()
    }
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }
}

// Wasm-compatible synchronous algorithm execution
#[cfg(target_arch = "wasm32")]
impl ReadCtx {
    /// Execute the selected algorithm synchronously (for wasm)
    pub fn run_algorithm_sync(
        &mut self,
        algorithm: Algorithm,
        cancelled: &Arc<AtomicBool>,
    ) {
        web_sys::console::log_1(
            &format!("Running algorithm: {:?}", algorithm).into(),
        );

        match algorithm {
            Algorithm::NgramsParseCorpus => {
                web_sys::console::log_1(
                    &"Dispatching to run_ngrams_parse_corpus_sync".into(),
                );
                self.run_ngrams_parse_corpus_sync(cancelled);
            },
            Algorithm::ContextRead => {
                web_sys::console::log_1(
                    &"Dispatching to run_context_read_sync".into(),
                );
                self.run_context_read_sync(cancelled);
            },
            Algorithm::ContextInsert => {
                web_sys::console::log_1(
                    &"Dispatching to run_context_insert_sync".into(),
                );
                self.run_context_insert_sync(cancelled);
            },
        }

        web_sys::console::log_1(&"Task done.".into());
    }

    /// Run ngrams::parse_corpus algorithm synchronously
    fn run_ngrams_parse_corpus_sync(
        &mut self,
        cancelled: &Arc<AtomicBool>,
    ) {
        web_sys::console::log_1(
            &"Starting run_ngrams_parse_corpus_sync...".into(),
        );

        use ngrams::{
            cancellation::Cancellation,
            graph::{
                parse_corpus,
                traversal::pass::CancelReason,
                Corpus,
                ParseResult,
                Status,
                StatusHandle,
            },
        };
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{
                Hash,
                Hasher,
            },
        };

        let graph = self.graph.graph.clone();
        let labels = self.graph.labels.clone();
        let insert_texts = self.graph.insert_texts.clone();

        web_sys::console::log_1(
            &format!("Insert texts: {:?}", insert_texts).into(),
        );

        let status = StatusHandle::from(Status::new(insert_texts.clone()));
        self.status = Some(status.clone());

        let corpus_name = {
            let mut hasher = DefaultHasher::new();
            insert_texts.hash(&mut hasher);
            format!("{:?}", hasher.finish())
        };
        let corpus = Corpus::new(corpus_name.clone(), insert_texts);

        web_sys::console::log_1(
            &format!("Created corpus: {}", corpus_name).into(),
        );

        // Create a wasm-compatible cancellation using the Arc<AtomicBool>
        let cancellation = Cancellation::from(cancelled.clone());

        web_sys::console::log_1(&"Calling parse_corpus...".into());

        // Run the sync parse_corpus
        let result = parse_corpus(corpus, status, cancellation);

        web_sys::console::log_1(&"parse_corpus returned".into());

        match result {
            Ok(res) => {
                self.graph.insert_texts.clear();
                *graph.write().unwrap() = res.graph.into();
                *labels.write().unwrap() = res.labels;
                web_sys::console::log_1(
                    &"Ngrams parse corpus completed successfully".into(),
                );
            },
            Err(CancelReason::Cancelled) => {
                web_sys::console::log_1(
                    &"Parse operation was cancelled via token".into(),
                );
            },
            Err(CancelReason::Error) => {
                web_sys::console::error_1(
                    &"Parse operation encountered an error".into(),
                );
            },
        }
    }

    /// Run context-read algorithm synchronously
    fn run_context_read_sync(
        &mut self,
        cancelled: &Arc<AtomicBool>,
    ) {
        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        let combined_text: String = insert_texts.join("");

        if combined_text.is_empty() {
            web_sys::console::log_1(&"No text to read".into());
            return;
        }

        let mut read_ctx = context_read::context::ReadCtx::new(
            graph_ref.clone(),
            combined_text.chars(),
        );

        // Process the sequence
        while !cancelled.load(Ordering::SeqCst) {
            if read_ctx.next().is_none() {
                break;
            }
        }

        if cancelled.load(Ordering::SeqCst) {
            web_sys::console::log_1(
                &"Context read operation was cancelled".into(),
            );
        } else {
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            web_sys::console::log_1(
                &"Context read completed successfully".into(),
            );
        }
    }

    /// Run context-insert algorithm synchronously
    fn run_context_insert_sync(
        &mut self,
        cancelled: &Arc<AtomicBool>,
    ) {
        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        let mut insert_ctx =
            context_insert::InsertCtx::<Token>::from(graph_ref.clone());

        for text in &insert_texts {
            if cancelled.load(Ordering::SeqCst) {
                web_sys::console::log_1(
                    &"Context insert operation was cancelled".into(),
                );
                return;
            }

            if text.is_empty() {
                continue;
            }

            // Use new_atom_indices to insert atoms that don't exist yet
            let atom_indices = graph_ref.new_atom_indices(text.chars());
            let tokens: Vec<Token> = atom_indices
                .into_iter()
                .map(|idx| Token::new(idx.vertex_index(), 1))
                .collect();

            match insert_ctx.insert(tokens) {
                Ok(_result) => {
                    web_sys::console::log_1(
                        &format!("Inserted text: {}", text).into(),
                    );
                },
                Err(err) => {
                    web_sys::console::log_1(
                        &format!("Failed to insert text '{}': {:?}", text, err)
                            .into(),
                    );
                },
            }
        }

        if !cancelled.load(Ordering::SeqCst) {
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            web_sys::console::log_1(
                &"Context insert completed successfully".into(),
            );
        }
    }
}

// Wasm-compatible async algorithm execution
#[cfg(target_arch = "wasm32")]
impl ReadCtx {
    /// Execute the selected algorithm asynchronously (for wasm)
    /// This version uses async/await and yields to the event loop
    pub async fn run_algorithm_async(
        &mut self,
        algorithm: Algorithm,
        cancellation: crate::task::CancellationHandle,
    ) {
        use gloo_timers::future::TimeoutFuture;

        web_sys::console::log_1(
            &format!("Running async algorithm: {:?}", algorithm).into(),
        );

        match algorithm {
            Algorithm::NgramsParseCorpus => {
                self.run_ngrams_parse_corpus_async(&cancellation).await;
            },
            Algorithm::ContextRead => {
                self.run_context_read_async(&cancellation).await;
            },
            Algorithm::ContextInsert => {
                self.run_context_insert_async(&cancellation).await;
            },
        }

        web_sys::console::log_1(&"Async task done.".into());
    }

    /// Run ngrams::parse_corpus algorithm asynchronously
    async fn run_ngrams_parse_corpus_async(
        &mut self,
        cancellation: &crate::task::CancellationHandle,
    ) {
        use gloo_timers::future::TimeoutFuture;
        use ngrams::{
            cancellation::Cancellation,
            graph::{
                parse_corpus,
                traversal::pass::CancelReason,
                Corpus,
                Status,
                StatusHandle,
            },
        };
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{
                Hash,
                Hasher,
            },
        };

        web_sys::console::log_1(
            &"Starting async ngrams parse corpus...".into(),
        );

        let graph = self.graph.graph.clone();
        let labels = self.graph.labels.clone();
        let insert_texts = self.graph.insert_texts.clone();

        let status = StatusHandle::from(Status::new(insert_texts.clone()));
        self.status = Some(status.clone());

        let corpus_name = {
            let mut hasher = DefaultHasher::new();
            insert_texts.hash(&mut hasher);
            format!("{:?}", hasher.finish())
        };
        let corpus = Corpus::new(corpus_name.clone(), insert_texts);

        // Create a wasm-compatible cancellation
        let ngrams_cancellation = Cancellation::from(cancellation.flag());

        // Yield to allow UI to update
        TimeoutFuture::new(0).await;

        // Run the parse_corpus (this is still sync internally, but we yield before/after)
        let result =
            ngrams::graph::parse_corpus(corpus, status, ngrams_cancellation);

        match result {
            Ok(res) => {
                self.graph.insert_texts.clear();
                *graph.write().unwrap() = res.graph.into();
                *labels.write().unwrap() = res.labels;
                web_sys::console::log_1(
                    &"Ngrams parse corpus completed successfully".into(),
                );
            },
            Err(CancelReason::Cancelled) => {
                web_sys::console::log_1(
                    &"Parse operation was cancelled".into(),
                );
            },
            Err(CancelReason::Error) => {
                web_sys::console::error_1(
                    &"Parse operation encountered an error".into(),
                );
            },
        }
    }

    /// Run context-read algorithm asynchronously
    async fn run_context_read_async(
        &mut self,
        cancellation: &crate::task::CancellationHandle,
    ) {
        use gloo_timers::future::TimeoutFuture;

        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();
        let combined_text: String = insert_texts.join("");

        if combined_text.is_empty() {
            web_sys::console::log_1(&"No text to read".into());
            return;
        }

        let mut read_ctx = context_read::context::ReadCtx::new(
            graph_ref.clone(),
            combined_text.chars(),
        );

        // Yield before starting
        TimeoutFuture::new(0).await;

        // Process in chunks to allow UI to remain responsive
        let mut iteration = 0;
        while !cancellation.is_cancelled() {
            if read_ctx.next().is_none() {
                break;
            }

            iteration += 1;
            // Yield every 100 iterations to keep UI responsive
            if iteration % 100 == 0 {
                TimeoutFuture::new(0).await;
            }
        }

        if !cancellation.is_cancelled() {
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            web_sys::console::log_1(
                &"Context read completed successfully".into(),
            );
        }
    }

    /// Run context-insert algorithm asynchronously
    async fn run_context_insert_async(
        &mut self,
        cancellation: &crate::task::CancellationHandle,
    ) {
        use gloo_timers::future::TimeoutFuture;

        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        let mut insert_ctx =
            context_insert::InsertCtx::<Token>::from(graph_ref.clone());

        // Yield before starting
        TimeoutFuture::new(0).await;

        for (i, text) in insert_texts.iter().enumerate() {
            if cancellation.is_cancelled() {
                web_sys::console::log_1(&"Context insert cancelled".into());
                return;
            }

            if text.is_empty() {
                continue;
            }

            web_sys::console::log_1(
                &format!("Inserting text {}: {}", i + 1, text).into(),
            );

            // Use new_atom_indices to insert atoms that don't exist yet
            let atom_indices = graph_ref.new_atom_indices(text.chars());
            let tokens: Vec<Token> = atom_indices
                .into_iter()
                .map(|idx| Token::new(idx.vertex_index(), 1))
                .collect();

            match insert_ctx.insert(tokens) {
                Ok(_result) => {
                    web_sys::console::log_1(
                        &format!("Inserted: {}", text).into(),
                    );
                },
                Err(err) => {
                    web_sys::console::error_1(
                        &format!("Error inserting '{}': {:?}", text, err)
                            .into(),
                    );
                },
            }

            // Yield after each insert to keep UI responsive
            TimeoutFuture::new(0).await;
        }

        if !cancellation.is_cancelled() {
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            web_sys::console::log_1(
                &"Context insert completed successfully".into(),
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ReadCtx {
    /// Execute the selected algorithm on the input texts
    pub async fn run_algorithm(
        &mut self,
        algorithm: Algorithm,
        cancellation_token: CancellationToken,
    ) {
        println!("Task running on thread {:?}", std::thread::current().id());
        println!("Running algorithm: {:?}", algorithm);

        match algorithm {
            Algorithm::NgramsParseCorpus => {
                self.run_ngrams_parse_corpus(cancellation_token).await;
            },
            Algorithm::ContextRead => {
                self.run_context_read(cancellation_token).await;
            },
            Algorithm::ContextInsert => {
                self.run_context_insert(cancellation_token).await;
            },
        }

        println!("Task done.");
    }

    /// Run ngrams::parse_corpus algorithm
    async fn run_ngrams_parse_corpus(
        &mut self,
        cancellation_token: CancellationToken,
    ) {
        let graph = self.graph.graph.clone();
        let labels = self.graph.labels.clone();
        let insert_texts = self.graph.insert_texts.clone();

        let status = StatusHandle::from(Status::new(insert_texts.clone()));
        self.status = Some(status.clone());

        let corpus_name = {
            let mut hasher = DefaultHasher::new();
            insert_texts.hash(&mut hasher);
            format!("{:?}", hasher.finish())
        };
        let corpus = ngrams::graph::Corpus::new(corpus_name, insert_texts);

        // Convert the cancellation token to the ngrams Cancellation type
        let cancellation =
            ngrams::cancellation::Cancellation::from(cancellation_token);

        // Run the sync parse_corpus in a blocking task
        let res = tokio::task::spawn_blocking(move || {
            ngrams::graph::parse_corpus(corpus, status, cancellation)
        })
        .await;

        match res {
            Ok(Ok(res)) => {
                self.graph.insert_texts.clear();
                *graph.write().unwrap() = res.graph.into();
                *labels.write().unwrap() = res.labels;
            },
            Ok(Err(CancelReason::Cancelled)) => {
                println!("Parse operation was cancelled via token");
            },
            Ok(Err(CancelReason::Error)) => {
                println!("Parse operation panicked");
            },
            Err(join_error) => {
                println!("Parse task panicked: {:?}", join_error);
            },
        }
    }

    /// Run context-read::read algorithm
    async fn run_context_read(
        &mut self,
        cancellation_token: CancellationToken,
    ) {
        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        // Create a combined sequence from all insert texts
        let combined_text: String = insert_texts.join("");

        if combined_text.is_empty() {
            println!("No text to read");
            return;
        }

        // Create a ReadCtx from context_read and run it
        // The ReadCtx::new takes a HypergraphRef and something implementing ToNewAtomIndices (like Chars)
        let mut read_ctx = context_read::context::ReadCtx::new(
            graph_ref.clone(),
            combined_text.chars(),
        );

        // Process the sequence - iterate through blocks with cancellation check
        while !cancellation_token.is_cancelled() {
            if read_ctx.next().is_none() {
                break;
            }
        }

        if cancellation_token.is_cancelled() {
            println!("Context read operation was cancelled");
        } else {
            // Update the graph with the result
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            println!("Context read completed successfully");
        }
    }

    /// Run context-insert::insert algorithm
    async fn run_context_insert(
        &mut self,
        cancellation_token: CancellationToken,
    ) {
        let graph_ref: HypergraphRef = self.graph.read().clone();
        let insert_texts = self.graph.insert_texts.clone();

        // Create an InsertCtx and insert each text
        let mut insert_ctx =
            context_insert::InsertCtx::<Token>::from(graph_ref.clone());

        for text in &insert_texts {
            if cancellation_token.is_cancelled() {
                println!("Context insert operation was cancelled");
                return;
            }

            if text.is_empty() {
                continue;
            }

            // Convert text characters to tokens using the graph
            // First, get atom children for the characters
            let tokens = graph_ref.expect_atom_children(text.chars());

            match insert_ctx.insert(tokens) {
                Ok(_result) => {
                    println!("Inserted text: {}", text);
                },
                Err(err) => {
                    println!("Failed to insert text '{}': {:?}", text, err);
                },
            }
        }

        if !cancellation_token.is_cancelled() {
            // Update the graph with the result
            *self.graph.write() = graph_ref;
            self.graph.insert_texts.clear();
            println!("Context insert completed successfully");
        }
    }
}
