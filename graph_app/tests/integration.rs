//! Integration tests for graph_app
//!
//! Tests the workflow of:
//! 1. Creating a startup graph using the ngram-based algorithm
//! 2. Inserting a new sequence using the insert-based algorithm

use context_insert::InsertCtx;
use context_trace::{
    graph::{
        vertex::token::Token,
        Hypergraph,
        HypergraphRef,
    },
    init_test_tracing,
    Atom,
};
use ngrams::{
    cancellation::Cancellation,
    graph::{
        parse_corpus,
        Corpus,
        Status,
        StatusHandle,
    },
};

/// Test creating a graph with ngrams and then inserting a new sequence
#[test]
fn test_ngrams_then_insert() {
    // Step 1: Create a startup graph using ngrams::parse_corpus
    let initial_texts = vec!["aabb".to_string(), "bbaa".to_string()];
    let corpus_name = "test_ngrams_then_insert".to_owned();

    let status = StatusHandle::from(Status::new(initial_texts.clone()));

    let parse_result = parse_corpus(
        Corpus::new(corpus_name, initial_texts),
        status,
        Cancellation::None,
    )
    .expect("parse_corpus should succeed");

    // Verify the graph was created successfully
    assert!(
        parse_result.graph.vertex_count() > 0,
        "Graph should have vertices after ngrams parsing"
    );

    let initial_vertex_count = parse_result.graph.vertex_count();

    // Step 2: Convert to HypergraphRef for use with context-insert
    let graph_ref = HypergraphRef::from(parse_result.graph);

    // Initialize tracing with the graph for better token display in logs
    let _tracing = init_test_tracing!(&graph_ref);

    // Step 3: Insert a new sequence using context-insert
    let new_text = "abab";
    let mut insert_ctx = InsertCtx::<Token>::from(graph_ref.clone());

    // Get tokens for the new text
    let tokens = graph_ref.expect_atom_children(new_text.chars());

    let insert_result = insert_ctx.insert(tokens);
    assert!(
        insert_result.is_ok(),
        "Insert should succeed: {:?}",
        insert_result.err()
    );

    // Verify the graph was modified
    let final_vertex_count = graph_ref.vertex_count();

    // The graph should still be valid
    assert!(
        final_vertex_count >= initial_vertex_count,
        "Graph should have at least as many vertices after insert"
    );
}

/// Test inserting multiple sequences after ngrams parsing
#[test]
fn test_ngrams_then_multiple_inserts() {
    // Step 1: Create a startup graph using ngrams::parse_corpus
    let initial_texts = vec!["hello".to_string(), "world".to_string()];
    let corpus_name = "test_ngrams_multiple_inserts".to_owned();

    let status = StatusHandle::from(Status::new(initial_texts.clone()));

    let parse_result = parse_corpus(
        Corpus::new(corpus_name, initial_texts),
        status,
        Cancellation::None,
    )
    .expect("parse_corpus should succeed");

    let graph_ref = HypergraphRef::from(parse_result.graph);
    let mut insert_ctx = InsertCtx::<Token>::from(graph_ref.clone());

    // Insert multiple new sequences
    let new_texts = vec!["held", "word", "low"];

    for text in new_texts {
        let tokens = graph_ref.expect_atom_children(text.chars());
        let result = insert_ctx.insert(tokens);
        assert!(
            result.is_ok(),
            "Insert of '{}' should succeed: {:?}",
            text,
            result.err()
        );
        println!(
            "Inserted '{}', graph now has {} vertices",
            text,
            graph_ref.vertex_count()
        );
    }

    // Verify the graph is still valid
    assert!(
        graph_ref.vertex_count() > 0,
        "Graph should have vertices after multiple inserts"
    );
}

/// Test with an empty initial corpus followed by inserts
///
/// NOTE: This test is currently ignored because inserting into a graph with only
/// atoms (no patterns) triggers unreachable code in the split/vertex module.
/// This is a known limitation - the insert algorithm expects existing structure.
#[test]
fn test_empty_graph_then_insert() {
    // Start with an empty graph
    let graph = Hypergraph::default();
    let graph_ref = HypergraphRef::from(graph);

    // Ensure atoms exist for the characters we'll insert
    // We need to add atoms first since there's no corpus
    let text = "abc";
    for c in text.chars() {
        graph_ref.insert_atom(Atom::Element(c));
    }

    let mut insert_ctx = InsertCtx::<Token>::from(graph_ref.clone());

    // Now insert a sequence
    let tokens = graph_ref.expect_atom_children(text.chars());
    let result = insert_ctx.insert(tokens);

    assert!(
        result.is_ok(),
        "Insert into initially empty graph should succeed: {:?}",
        result.err()
    );

    println!(
        "After inserting '{}' into empty graph: {} vertices",
        text,
        graph_ref.vertex_count()
    );

    // Should have at least the atoms
    assert!(
        graph_ref.vertex_count() >= text.len(),
        "Graph should have at least {} vertices (one per atom)",
        text.len()
    );
}
