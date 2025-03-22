// Declare our modules
pub mod analyzer;
pub mod formatter;
pub mod cli;

// Re-export key types for convenience
pub use analyzer::{LogAnalyzer, AnalysisResult};
pub use formatter::print_results;
pub use cli::Args;