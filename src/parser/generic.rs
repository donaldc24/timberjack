use super::{LogParser, ParsedLogLine};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LEVEL_REGEX: Regex = Regex::new(
        r"\[((?i)ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE)]|(?i:ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE):"
    ).unwrap();

    static ref TIMESTAMP_REGEX: Regex = Regex::new(
        r"(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})"
    ).unwrap();
}

/// Generic log parser that works with standard log formats
pub struct GenericLogParser;

impl LogParser for GenericLogParser {
    fn name(&self) -> &'static str {
        "Generic"
    }

    fn can_parse(&self, _sample_lines: &[&str]) -> bool {
        // Generic parser can handle any log format as a fallback
        true
    }

    fn parse_line(&self, line: &str) -> ParsedLogLine {
        let mut parsed = ParsedLogLine::default();

        // Always set the full line as the default message
        parsed.message = Some(line.to_string());

        // Extract log level
        if let Some(caps) = LEVEL_REGEX.captures(line) {
            parsed.level = caps.get(1)
                .map_or_else(
                    || caps.get(0).map(|m| m.as_str().to_uppercase()),
                    |m| Some(m.as_str().to_uppercase())
                )
                .map(|s| s.to_string());
        }

        // Extract timestamp
        if let Some(caps) = TIMESTAMP_REGEX.captures(line) {
            if let Some(m) = caps.get(1) {
                let ts = m.as_str();
                parsed.timestamp = Some(
                    if ts.len() >= 13 {
                        ts[0..13].to_string()
                    } else {
                        ts.to_string()
                    }
                );
            }
        }

        parsed
    }
}
