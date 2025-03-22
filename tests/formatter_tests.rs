use std::collections::{HashMap, HashSet};
use timber_rs::analyzer::AnalysisResult;

#[cfg(test)]
use timber_rs::formatter::print_results_to_writer;

#[test]
fn test_print_basic_results() {
    // Create a mock AnalysisResult
    let mut result = AnalysisResult {
        matched_lines: vec![
            "2025-03-21 14:00:00,123 [ERROR] NullPointerException".to_string(),
            "2025-03-21 14:03:00,012 [ERROR] Connection timeout".to_string(),
        ],
        count: 2,
        time_trends: HashMap::new(),
        levels_count: HashMap::new(),
        error_types: HashMap::new(),
        unique_messages: HashSet::new(),
    };

    // Add some level counts
    result.levels_count.insert("ERROR".to_string(), 2);

    // Capture the output to test
    let mut output = Vec::new();
    print_results_to_writer(&result, false, false, &mut output).unwrap();

    // Check the output
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("NullPointerException"));
    assert!(output_str.contains("Connection timeout"));
    assert!(output_str.contains("Felled: 2 logs"));
}

#[test]
fn test_print_trend_results() {
    // Create a mock AnalysisResult with time trends
    let mut result = AnalysisResult {
        matched_lines: vec![
            "2025-03-21 14:00:00,123 [ERROR] NullPointerException".to_string(),
            "2025-03-21 15:03:00,012 [ERROR] Connection timeout".to_string(),
        ],
        count: 2,
        time_trends: HashMap::new(),
        levels_count: HashMap::new(),
        error_types: HashMap::new(),
        unique_messages: HashSet::new(),
    };

    // Add time trends
    result.time_trends.insert("2025-03-21 14".to_string(), 1);
    result.time_trends.insert("2025-03-21 15".to_string(), 1);

    // Capture the output to test
    let mut output = Vec::new();
    print_results_to_writer(&result, true, false, &mut output).unwrap();

    // Check the output
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Time trends:"));
    assert!(output_str.contains("2025-03-21 14 - 1 log occurred"));
    assert!(output_str.contains("2025-03-21 15 - 1 log occurred"));
}

#[test]
fn test_print_stats_results() {
    // Create a mock AnalysisResult with stats
    let mut result = AnalysisResult {
        matched_lines: vec![
            "2025-03-21 14:00:00,123 [ERROR] ServiceA - NullPointerException".to_string(),
            "2025-03-21 14:01:00,456 [WARN] ServiceB - Slow database query".to_string(),
            "2025-03-21 14:03:00,012 [ERROR] ServiceC - Connection timeout".to_string(),
        ],
        count: 3,
        time_trends: HashMap::new(),
        levels_count: HashMap::new(),
        error_types: HashMap::new(),
        unique_messages: HashSet::new(),
    };

    // Add level counts
    result.levels_count.insert("ERROR".to_string(), 2);
    result.levels_count.insert("WARN".to_string(), 1);

    // Add error types
    result
        .error_types
        .insert("NullPointerException".to_string(), 1);
    result
        .error_types
        .insert("Connection timeout".to_string(), 1);

    // Add unique messages
    result
        .unique_messages
        .insert("NullPointerException".to_string());
    result
        .unique_messages
        .insert("Slow database query".to_string());
    result
        .unique_messages
        .insert("Connection timeout".to_string());

    // Capture the output to test
    let mut output = Vec::new();
    print_results_to_writer(&result, false, true, &mut output).unwrap();

    // Check the output
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Stats summary:"));
    assert!(output_str.contains("Log levels:"));
    assert!(output_str.contains("ERROR: 2 logs"));
    assert!(output_str.contains("WARN: 1 log"));
    assert!(output_str.contains("Top error types:"));
    assert!(output_str.contains("NullPointerException: 1 occurrence"));
    assert!(output_str.contains("Connection timeout: 1 occurrence"));
    assert!(output_str.contains("Unique messages: 3"));
    assert!(output_str.contains("Repetition ratio: 0.0%"));
}

#[test]
fn test_empty_results() {
    let result = AnalysisResult {
        matched_lines: vec![],
        count: 0,
        time_trends: HashMap::new(),
        levels_count: HashMap::new(),
        error_types: HashMap::new(),
        unique_messages: HashSet::new(),
    };

    let mut output = Vec::new();
    print_results_to_writer(&result, true, true, &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Felled: 0 logs"));
    assert!(!output_str.contains("Time trends:"));
    assert!(output_str.contains("Stats summary:"));
}
