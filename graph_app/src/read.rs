use ngrams::graph::{
    traversal::pass::CancelReason,
    Status,
    StatusHandle,
};
#[cfg(feature = "persistence")]
use serde::*;

#[allow(unused)]
use crate::graph::*;

use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};
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
    pub async fn read_text(
        &mut self,
        cancellation_token: CancellationToken,
    ) {
        println!("Task running on thread {:?}", std::thread::current().id());

        let graph = self.graph.graph.clone();
        let labels = self.graph.labels.clone();
        let insert_texts = self.graph.insert_texts.clone();

        let status = StatusHandle::from(Status::new(insert_texts.clone()));
        self.status = Some(status.clone());
        //let corpus_name = "7547453137468837744".to_string();
        let corpus_name = {
            let mut hasher = DefaultHasher::new();
            insert_texts.hash(&mut hasher);
            format!("{:?}", hasher.finish())
        };
        let corpus = ngrams::graph::Corpus::new(corpus_name, insert_texts);

        // Parse has periodic cancellation checks during the parse operation
        // Use select to race between parsing and cancellation
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

        println!("Task done.");
    }
}
