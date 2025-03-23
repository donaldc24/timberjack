use regex::Regex;
use timber_rs::analyzer::LogAnalyzer;

#[test]
fn test_analyze_line_with_pattern() {
    let analyzer = LogAnalyzer::new();
    let line = "2025-03-21 14:00:00,123 [ERROR] NullPointerException";
    let pattern = Some(Regex::new("NullPointer").unwrap());

    let result = analyzer.analyze_line(line, pattern.as_ref(), None, false, false);
    assert!(result.is_some());

    let (matched, level, _) = result.unwrap();
    assert_eq!(matched, line);
    assert_eq!(level, "ERROR");
}

#[test]
fn test_analyze_line_with_level_filter() {
    let analyzer = LogAnalyzer::new();
    let line = "2025-03-21 14:00:00,123 [ERROR] NullPointerException";

    // Should match ERROR level
    let result = analyzer.analyze_line(line, None, Some("ERROR"), false, false);
    assert!(result.is_some());

    // Should not match WARN level
    let result = analyzer.analyze_line(line, None, Some("WARN"), false, false);
    assert!(result.is_none());
}

#[test]
fn test_extract_error_type() {
    let analyzer = LogAnalyzer::new();

    // Test exception extraction
    let line = "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42";
    let error_type = analyzer.extract_error_type(line);
    assert_eq!(error_type, Some("NullPointerException".to_string()));

    // Test timeout extraction
    let line = "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86";
    let error_type = analyzer.extract_error_type(line);
    assert_eq!(error_type, Some("Connection timeout".to_string()));
}

#[test]
fn test_analyze_lines_with_pattern() {
    let mut analyzer = LogAnalyzer::new();
    let lines = vec![
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42".to_string(),
        "2025-03-21 14:01:00,456 [WARN] Slow database query (2.3s) in DatabaseService.java:128"
            .to_string(),
        "2025-03-21 14:02:00,789 [INFO] Application started successfully".to_string(),
        "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86".to_string(),
    ];

    // Search for ERROR pattern
    let pattern = Some(Regex::new("ERROR").unwrap());
    let result = analyzer.analyze_lines(lines.into_iter(), pattern.as_ref(), None, false, false);

    assert_eq!(result.count, 2);
    assert_eq!(result.matched_lines.len(), 2);
    assert!(result.matched_lines[0].contains("NullPointerException"));
    assert!(result.matched_lines[1].contains("Connection timeout"));
}

#[test]
fn test_analyze_lines_with_level_filter() {
    let mut analyzer = LogAnalyzer::new();
    let lines = vec![
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42".to_string(),
        "2025-03-21 14:01:00,456 [WARN] Slow database query (2.3s) in DatabaseService.java:128"
            .to_string(),
        "2025-03-21 14:02:00,789 [INFO] Application started successfully".to_string(),
        "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86".to_string(),
    ];

    // Filter by WARN level
    let result = analyzer.analyze_lines(lines.into_iter(), None, Some("WARN"), false, false);

    assert_eq!(result.count, 1);
    assert_eq!(result.matched_lines.len(), 1);
    assert!(result.matched_lines[0].contains("Slow database query"));
}

#[test]
fn test_time_trends() {
    let mut analyzer = LogAnalyzer::new();
    let lines = vec![
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42".to_string(),
        "2025-03-21 14:01:00,456 [WARN] Slow database query (2.3s) in DatabaseService.java:128"
            .to_string(),
        "2025-03-21 14:02:00,789 [INFO] Application started successfully".to_string(),
        "2025-03-21 15:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86".to_string(),
    ];

    // Collect time trends
    let result = analyzer.analyze_lines(lines.into_iter(), None, None, true, false);

    assert_eq!(result.time_trends.len(), 2);
    assert_eq!(*result.time_trends.get("2025-03-21 14").unwrap_or(&0), 3);
    assert_eq!(*result.time_trends.get("2025-03-21 15").unwrap_or(&0), 1);
}

#[test]
fn test_stats_collection() {
    let mut analyzer = LogAnalyzer::new();
    let lines = vec![
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42".to_string(),
        "2025-03-21 14:01:00,456 [WARN] Slow database query (2.3s) in DatabaseService.java:128"
            .to_string(),
        "2025-03-21 14:02:00,789 [INFO] Application started successfully".to_string(),
        "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86".to_string(),
    ];

    // Collect stats
    let result = analyzer.analyze_lines(lines.into_iter(), None, None, false, true);

    // Check level counts
    assert_eq!(result.levels_count.len(), 3);
    assert_eq!(*result.levels_count.get("ERROR").unwrap_or(&0), 2);
    assert_eq!(*result.levels_count.get("WARN").unwrap_or(&0), 1);
    assert_eq!(*result.levels_count.get("INFO").unwrap_or(&0), 1);

    // Check error types
    assert_eq!(result.error_types.len(), 2);
    assert_eq!(
        *result.error_types.get("NullPointerException").unwrap_or(&0),
        1
    );
    assert_eq!(
        *result.error_types.get("Connection timeout").unwrap_or(&0),
        1
    );

    // Check uniqueness
    assert_eq!(result.unique_messages.len(), 4);
}

#[test]
fn test_combined_filters() {
    let mut analyzer = LogAnalyzer::new();
    let lines = vec![
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42".to_string(),
        "2025-03-21 14:01:00,456 [WARN] Slow database query (2.3s) in DatabaseService.java:128"
            .to_string(),
        "2025-03-21 14:02:00,789 [INFO] Application started successfully".to_string(),
        "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86".to_string(),
    ];

    // Pattern + level filter
    let pattern = Some(Regex::new("Connection").unwrap());
    let result = analyzer.analyze_lines(
        lines.into_iter(),
        pattern.as_ref(),
        Some("ERROR"),
        false,
        false,
    );

    assert_eq!(result.count, 1);
    assert!(result.matched_lines[0].contains("Connection timeout"));
}

#[test]
fn test_empty_results() {
    let mut analyzer = LogAnalyzer::new();
    let lines = vec![
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42".to_string(),
        "2025-03-21 14:01:00,456 [WARN] Slow database query (2.3s) in DatabaseService.java:128"
            .to_string(),
    ];

    // Pattern that doesn't match anything
    let pattern = Some(Regex::new("ThisDoesNotExist").unwrap());
    let result = analyzer.analyze_lines(lines.into_iter(), pattern.as_ref(), None, false, false);

    assert_eq!(result.count, 0);
    assert!(result.matched_lines.is_empty());
}