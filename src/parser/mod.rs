use std::sync::Arc;

/// Trait defining the interface for all log format parsers
pub trait LogParser: Send + Sync {
    /// Returns the name of the parser
    fn name(&self) -> &'static str;

    /// Checks if this parser can handle the given log format
    /// based on sample lines
    fn can_parse(&self, sample_lines: &[&str]) -> bool;

    /// Parses a single line into structured data
    fn parse_line(&self, line: &str) -> ParsedLogLine;
}

/// Structured representation of a parsed log line
#[derive(Debug, Clone, Default)]
pub struct ParsedLogLine {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub fields: std::collections::HashMap<String, String>,
}

/// Log format types supported by Timber
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Generic,
    Json,
    Apache,
    Syslog,
    // More formats can be added later
}

/// Registry for managing parsers and detecting formats
pub struct ParserRegistry {
    parsers: Vec<(LogFormat, Arc<dyn LogParser>)>,
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserRegistry {
    /// Create a new parser registry with default parsers
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: Vec::new(),
        };

        // Register the generic parser by default
        let generic_parser = Arc::new(generic::GenericLogParser);
        registry.register_parser(LogFormat::Generic, generic_parser);

        // Register the JSON parser
        let json_parser = Arc::new(json::JsonLogParser);
        registry.register_parser(LogFormat::Json, json_parser);

        registry
    }

    /// Register a new parser
    pub fn register_parser(&mut self, format: LogFormat, parser: Arc<dyn LogParser>) {
        self.parsers.push((format, parser));
    }

    /// Detect format from sample lines
    pub fn detect_format(&self, sample_lines: &[&str]) -> (LogFormat, Arc<dyn LogParser>) {
        // First, try sampling more lines if possible
        let sample = if sample_lines.len() < 5 {
            sample_lines
        } else {
            &sample_lines[..5] // Take first 5 lines for detection
        };

        // First, try JSON parser explicitly for more common JSON log scenarios
        let json_parser = self.get_parser(LogFormat::Json).unwrap();
        if json_parser.can_parse(sample) {
            return (LogFormat::Json, json_parser);
        }

        // If JSON detection fails, try other parsers
        for (format, parser) in &self.parsers {
            if *format != LogFormat::Json && parser.can_parse(sample) {
                return (*format, parser.clone());
            }
        }

        // Fallback to generic parser if no match
        (
            LogFormat::Generic,
            self.get_parser(LogFormat::Generic).unwrap(),
        )
    }

    /// Get parser for specific format
    pub fn get_parser(&self, format: LogFormat) -> Option<Arc<dyn LogParser>> {
        self.parsers
            .iter()
            .find(|(f, _)| *f == format)
            .map(|(_, p)| p.clone())
    }
}

// Create submodules for specific parsers
pub mod generic;
pub mod json;
