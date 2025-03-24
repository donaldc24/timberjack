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

        let valid_count = sample_lines.iter()
            .filter(|line| {
                let trimmed = line.trim();
                (trimmed.starts_with('{') && trimmed.ends_with('}')) &&
                    serde_json::from_str::<Value>(trimmed).is_ok()
            })
            .count();

        // Require at least 60% of lines to be valid JSON
        valid_count * 100 / sample_lines.len() >= 60
    }

    fn parse_line<'a>(&self, line: &'a str) -> ParsedLogLine<'a> {
        let mut parsed = ParsedLogLine::default();
        parsed.message = Some(line);

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<Value>(line.trim()) {
            if let Some(obj) = json.as_object() {
                // Extract common timestamp fields
                for key in &["timestamp", "time", "@timestamp", "date", "datetime"] {
                    if let Some(ts) = obj.get(*key) {
                        if let Some(ts_str) = ts.as_str() {
                            parsed.timestamp = Some(ts_str);
                            // Store in fields too
                            parsed.fields.insert(key.to_string(), ts_str.to_string());
                            break;
                        }
                    }
                }

                // Extract common level fields
                for key in &["level", "severity", "loglevel", "log_level", "@level"] {
                    if let Some(lvl) = obj.get(*key) {
                        if let Some(lvl_str) = lvl.as_str() {
                            parsed.level = Some(lvl_str);
                            // Store in fields too
                            parsed.fields.insert(key.to_string(), lvl_str.to_string());
                            break;
                        }
                    }
                }

                // Extract message field
                for key in &["message", "msg", "text", "description", "content"] {
                    if let Some(msg) = obj.get(*key) {
                        if let Some(msg_str) = msg.as_str() {
                            // Keep original as well but set specific message field
                            parsed.message = Some(msg_str);
                            // Store in fields too
                            parsed.fields.insert(key.to_string(), msg_str.to_string());
                            break;
                        }
                    }
                }

                // Extract all fields (flattened for first level)
                for (key, value) in obj {
                    // Only handle string, number, and bool values
                    match value {
                        Value::String(s) => {
                            parsed.fields.insert(key.clone(), s.clone());
                        },
                        Value::Number(n) => {
                            parsed.fields.insert(key.clone(), n.to_string());
                        },
                        Value::Bool(b) => {
                            parsed.fields.insert(key.clone(), b.to_string());
                        },
                        _ => {} // Ignore other types for now
                    }
                }
            }
        }

        parsed
    }
}