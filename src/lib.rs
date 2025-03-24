// Declare our modules
pub mod accelerated;
pub mod analyzer;
pub mod cli;
pub mod formatter;
pub mod parser;

// Re-export key types for convenience
pub use analyzer::{AnalysisResult, LogAnalyzer};
pub use cli::Args;
pub use formatter::print_results;
pub use parser::{LogFormat, LogParser, ParserRegistry};
