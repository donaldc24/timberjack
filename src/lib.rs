// Declare our modules
pub mod analyzer;
pub mod cli;
pub mod formatter;

// Re-export key types for convenience
pub use analyzer::{AnalysisResult, LogAnalyzer};
pub use cli::Args;
pub use formatter::print_results;
