use strum::{
    Display,
    EnumIter,
};

/// Available algorithms for processing text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter)]
pub enum Algorithm {
    /// ngrams::parse_corpus - Parse corpus using n-gram frequency analysis
    #[default]
    #[strum(serialize = "N-grams Parse Corpus")]
    NgramsParseCorpus,

    /// context-read::read - Read sequences into the graph
    #[strum(serialize = "Context Read")]
    ContextRead,

    /// context-insert::insert - Insert patterns into the graph
    #[strum(serialize = "Context Insert")]
    ContextInsert,
}

impl Algorithm {
    /// Returns a description of what the algorithm does
    pub fn description(&self) -> &'static str {
        match self {
            Algorithm::NgramsParseCorpus => {
                "Parse corpus using n-gram frequency analysis and build a hypergraph"
            }
            Algorithm::ContextRead => {
                "Read sequences into the graph using context-read module"
            }
            Algorithm::ContextInsert => {
                "Insert patterns into the graph using context-insert module"
            }
        }
    }
}
