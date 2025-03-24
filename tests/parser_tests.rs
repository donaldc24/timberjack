#[cfg(test)]
mod tests {
    use timber_rs::parser::{LogFormat, ParserRegistry};

    #[test]
    fn test_generic_parser() {
        let registry = ParserRegistry::new();
        let generic_parser = registry.get_parser(LogFormat::Generic).unwrap();

        let line = "2025-03-21 14:00:00,123 [ERROR] Test error message";
        let parsed = generic_parser.parse_line(line);

        assert_eq!(parsed.level, Some("ERROR".to_string()));
        assert_eq!(parsed.timestamp, Some("2025-03-21 14".to_string()));
        assert_eq!(parsed.message, Some(line.to_string()));
    }

    #[test]
    fn test_format_detection() {
        let registry = ParserRegistry::new();
        let sample_lines = &[
            "2025-03-21 14:00:00,123 [ERROR] Test error message",
            "2025-03-21 14:01:00,456 [WARN] Test warning message",
        ];

        let (format, _) = registry.detect_format(sample_lines);
        assert_eq!(format, LogFormat::Generic);
    }
}
