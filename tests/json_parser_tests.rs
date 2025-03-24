#[cfg(test)]
mod tests {
    use timber_rs::parser::{ParserRegistry, LogParser, LogFormat};
    use std::sync::Arc;

    #[test]
    fn test_json_detection() {
        let registry = ParserRegistry::new();

        // Valid JSON logs
        let sample_lines = &[
            r#"{"timestamp":"2025-03-21T14:00:00.123Z","level":"ERROR","message":"Database connection failed"}"#,
            r#"{"time":"2025-03-21T14:01:00.456Z","severity":"WARN","msg":"Slow query detected"}"#
        ];

        let (format, _) = registry.detect_format(sample_lines);
        assert_eq!(format, LogFormat::Json);
    }

    #[test]
    fn test_json_parsing() {
        let registry = ParserRegistry::new();
        let json_parser = registry.get_parser(LogFormat::Json).unwrap();

        let line = r#"{"timestamp":"2025-03-21T14:00:00.123Z","level":"ERROR","message":"Database connection failed","service":"api","user_id":12345}"#;

        let parsed = json_parser.parse_line(line);

        assert_eq!(parsed.timestamp, Some("2025-03-21T14:00:00.123Z"));
        assert_eq!(parsed.level, Some("ERROR"));
        assert_eq!(parsed.message, Some("Database connection failed"));

        // Check fields
        assert!(parsed.fields.contains_key("timestamp"));
        assert!(parsed.fields.contains_key("level"));
        assert!(parsed.fields.contains_key("message"));
        assert!(parsed.fields.contains_key("service"));
        assert!(parsed.fields.contains_key("user_id"));
        assert_eq!(parsed.fields.get("service"), Some(&"api".to_string()));
        assert_eq!(parsed.fields.get("user_id"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_malformed_json() {
        let registry = ParserRegistry::new();
        let json_parser = registry.get_parser(LogFormat::Json).unwrap();

        // Malformed JSON should still produce a ParsedLogLine
        let line = r#"{"timestamp":"2025-03-21T14:00:00.123Z", "level":"ERROR", "message":"Missing closing brace"#;

        let parsed = json_parser.parse_line(line);

        // The message should be set to the entire line
        assert_eq!(parsed.message, Some(line));
        // But no fields should be extracted
        assert_eq!(parsed.timestamp, None);
        assert_eq!(parsed.level, None);
        assert!(parsed.fields.is_empty());
    }
}