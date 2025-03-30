// src/parser/json.rs - Corrected implementation

use super::{LogParser, ParsedLogLine};
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;

lazy_static! {
    // Timestamp keys to search for in JSON logs
    static ref TIMESTAMP_KEYS: Vec<&'static str> = vec!["timestamp", "time", "@timestamp", "date", "datetime"];
    // Level keys to search for in JSON logs
    static ref LEVEL_KEYS: Vec<&'static str> = vec!["level", "severity", "loglevel", "log_level", "@level"];
    // Message keys to search for in JSON logs
    static ref MESSAGE_KEYS: Vec<&'static str> = vec!["message", "msg", "text", "description", "content"];
}

/// Parser for JSON formatted logs
pub struct JsonLogParser;

impl Default for JsonLogParser {
    fn default() -> Self {
        JsonLogParser::new()
    }
}

impl JsonLogParser {
    /// Create a new JSON parser
    pub fn new() -> Self {
        Self {}
    }

    /// Extract a field from JSON using the first matching key
    fn find_first_value(&self, json: &Value, keys: &[&str]) -> Option<String> {
        if let Value::Object(obj) = json {
            for &key in keys {
                if let Some(value) = obj.get(key) {
                    match value {
                        Value::String(s) => return Some(s.clone()),
                        Value::Number(n) => return Some(n.to_string()),
                        Value::Bool(b) => return Some(b.to_string()),
                        _ => continue,
                    }
                }
            }
        }
        None
    }

    /// Extract all fields from a JSON object (including nested)
    fn extract_fields(json: &Value, prefix: &str, result: &mut HashMap<String, String>) {
        match json {
            Value::Object(map) => {
                for (key, value) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    match value {
                        Value::Object(_) | Value::Array(_) => {
                            if result.len() < 100 {
                                // Limit to prevent explosion
                                Self::extract_fields(value, &new_prefix, result);
                            }
                        }
                        Value::String(s) => {
                            result.insert(new_prefix, s.clone());
                        }
                        Value::Number(n) => {
                            result.insert(new_prefix, n.to_string());
                        }
                        Value::Bool(b) => {
                            result.insert(new_prefix, b.to_string());
                        }
                        Value::Null => {
                            result.insert(new_prefix, "null".to_string());
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    let new_prefix = format!("{}[{}]", prefix, i);
                    Self::extract_fields(item, &new_prefix, result);
                }
            }
            _ => {} // Not a container type, nothing to extract
        }
    }
}

impl LogParser for JsonLogParser {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn can_parse(&self, sample_lines: &[&str]) -> bool {
        // Check if lines look like valid JSON objects
        if sample_lines.is_empty() {
            return false;
        }

        let valid_count = sample_lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('{') &&
                    trimmed.ends_with('}') &&
                    serde_json::from_str::<Value>(trimmed).is_ok() &&
                    // Be more flexible about field names
                    (trimmed.contains("timestamp") ||
                        trimmed.contains("time") ||
                        trimmed.contains("@timestamp")) &&
                    (trimmed.contains("level") ||
                        trimmed.contains("severity") ||
                        trimmed.contains("log_level"))
            })
            .count();

        // Require at least 40% of lines to be valid JSON with somewhat relaxed field requirements
        valid_count * 100 / sample_lines.len() >= 40
    }

    fn parse_line(&self, line: &str) -> ParsedLogLine {
        let mut parsed = ParsedLogLine {
            message: Some(line.to_string()),
            ..Default::default()
        };

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<Value>(line.trim()) {
            // Try to extract timestamp
            parsed.timestamp = self.find_first_value(&json, &TIMESTAMP_KEYS);

            // Try to extract level
            parsed.level = self.find_first_value(&json, &LEVEL_KEYS);

            // Try to extract message
            parsed.message = self
                .find_first_value(&json, &MESSAGE_KEYS)
                .or_else(|| Some(line.to_string()));

            // Extract all fields for filtering
            JsonLogParser::extract_fields(&json, "", &mut parsed.fields);
        }

        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_detection() {
        let parser = JsonLogParser::new();

        // Valid JSON logs
        let sample_lines = &[
            r#"{"timestamp":"2025-03-21T14:00:00.123Z","level":"ERROR","message":"Database connection failed"}"#,
            r#"{"time":"2025-03-21T14:01:00.456Z","severity":"WARN","msg":"Slow query detected"}"#,
        ];

        assert!(parser.can_parse(sample_lines));

        // Invalid or non-JSON logs
        let non_json_lines = &[
            "2025-03-21 14:00:00,123 [ERROR] NullPointerException",
            "INFO: Application started at 14:00:00",
        ];

        assert!(!parser.can_parse(non_json_lines));
    }

    #[test]
    fn test_json_parsing() {
        let parser = JsonLogParser::new();
        let line = r#"{"timestamp":"2025-03-21T14:00:00.123Z","level":"ERROR","message":"Database connection failed","service":"api","user_id":12345}"#;

        let parsed = parser.parse_line(line);

        assert_eq!(
            parsed.timestamp,
            Some("2025-03-21T14:00:00.123Z".to_string())
        );
        assert_eq!(parsed.level, Some("ERROR".to_string()));
        assert_eq!(
            parsed.message,
            Some("Database connection failed".to_string())
        );

        // Check fields map
        assert!(parsed.fields.contains_key("timestamp"));
        assert!(parsed.fields.contains_key("level"));
        assert!(parsed.fields.contains_key("message"));
        assert!(parsed.fields.contains_key("service"));
        assert!(parsed.fields.contains_key("user_id"));
        assert_eq!(parsed.fields.get("service"), Some(&"api".to_string()));
        assert_eq!(parsed.fields.get("user_id"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_nested_json() {
        let parser = JsonLogParser::new();
        let line = r#"{"timestamp":"2025-03-21T14:00:00.123Z","level":"ERROR","user":{"id":"12345","name":"John"},"context":{"request":{"url":"/api/users"}}}"#;

        let parsed = parser.parse_line(line);

        // Check flattened nested fields
        assert!(parsed.fields.contains_key("user.id"));
        assert!(parsed.fields.contains_key("user.name"));
        assert!(parsed.fields.contains_key("context.request.url"));
        assert_eq!(parsed.fields.get("user.id"), Some(&"12345".to_string()));
        assert_eq!(parsed.fields.get("user.name"), Some(&"John".to_string()));
        assert_eq!(
            parsed.fields.get("context.request.url"),
            Some(&"/api/users".to_string())
        );
    }
}
