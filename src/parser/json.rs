use super::{LogParser, ParsedLogLine};
use serde_json::Value;

/// Parser for JSON formatted logs
pub struct JsonLogParser;

impl LogParser for JsonLogParser {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn can_parse(&self, sample_lines: &[&str]) -> bool {
        // Check if at least 60% of lines look like valid JSON
        if sample_lines.is_empty() {
            return false;
        }

        let valid_count = sample_lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                (trimmed.starts_with('{') && trimmed.ends_with('}'))
                    && serde_json::from_str::<Value>(trimmed).is_ok()
            })
            .count();

        // Require at least 60% of lines to be valid JSON
        valid_count * 100 / sample_lines.len() >= 60
    }

    fn parse_line<'a>(&self, line: &'a str) -> ParsedLogLine<'a> {
        let mut parsed = ParsedLogLine::default();

        // Always set the full line as the default message
        parsed.message = Some(line);

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<Value>(line.trim()) {
            if let Some(obj) = json.as_object() {
                // Fill the fields HashMap with owned String values
                for (key, value) in obj {
                    match value {
                        Value::String(s) => {
                            parsed.fields.insert(key.clone(), s.clone());
                        }
                        Value::Number(n) => {
                            parsed.fields.insert(key.clone(), n.to_string());
                        }
                        Value::Bool(b) => {
                            parsed.fields.insert(key.clone(), b.to_string());
                        }
                        _ => {} // Ignore other types for now
                    }
                }

                // Now the fields are collected - IMPORTANT: We must NOT use references from the json object!
                // Instead, we'll convert field values back to string slices from the original line

                // For timestamp: Check common fields and find in the original line
                for key in &["timestamp", "time", "@timestamp", "date", "datetime"] {
                    if let Some(json_str) = obj.get(*key).and_then(|v| v.as_str()) {
                        // Try to find this exact string in the original line
                        if let Some(pos) = line.find(json_str) {
                            parsed.timestamp = Some(&line[pos..pos + json_str.len()]);
                            break;
                        }
                    }
                }

                // For level: Check common fields and find in the original line
                for key in &["level", "severity", "loglevel", "log_level", "@level"] {
                    if let Some(json_str) = obj.get(*key).and_then(|v| v.as_str()) {
                        // Try to find this exact string in the original line
                        if let Some(pos) = line.find(json_str) {
                            parsed.level = Some(&line[pos..pos + json_str.len()]);
                            break;
                        }
                    }
                }

                // For message: Check common fields and find in the original line
                for key in &["message", "msg", "text", "description", "content"] {
                    if let Some(json_str) = obj.get(*key).and_then(|v| v.as_str()) {
                        // Try to find this exact string in the original line
                        if let Some(pos) = line.find(json_str) {
                            parsed.message = Some(&line[pos..pos + json_str.len()]);
                            break;
                        }
                    }
                }
            }
        }

        parsed
    }
}
