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

#[cfg(not(target_arch = "wasm32"))]
use context_trace::graph::HypergraphRef;
#[cfg(not(target_arch = "wasm32"))]
use context_trace::Token;
#[cfg(not(target_arch = "wasm32"))]
use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};
#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ReadCtx {
    graph: Graph,
    #[cfg(not(target_arch = "wasm32"))]
    status: Option<ngrams::graph::StatusHandle>,
}
impl ReadCtx {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            #[cfg(not(target_arch = "wasm32"))]
            status: None,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
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

        tokio::select! {
            res = ngrams::graph::parse_corpus(
                corpus,
                status,
                cancellation_token.clone(),
            ) => {
                match res {
                    Ok(res) => {
                        self.graph.insert_texts.clear();
                        *graph.write().unwrap() = res.graph.into();
                        *labels.write().unwrap() = res.labels;
                    },
                    Err(CancelReason::Cancelled) => {
                        println!("Parse operation was cancelled via token");
                    },
                    Err(CancelReason::Error) => {
                        println!("Parse operation panicked");
                    },
                }
            },
            _ = cancellation_token.cancelled() => {
                println!("Parse operation was cancelled via token during execution");
            }
        };
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
