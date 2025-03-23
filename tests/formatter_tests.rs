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
    print_results_to_writer(&result, false, false, &mut output, 5, false).unwrap();

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
    print_results_to_writer(&result, true, false, &mut output, 5, false).unwrap();

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
    print_results_to_writer(&result, false, true, &mut output, 5, false).unwrap();

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
    print_results_to_writer(&result, true, true, &mut output, 5, false).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Felled: 0 logs"));
    assert!(!output_str.contains("Time trends:"));
    assert!(output_str.contains("Stats summary:"));
}

#[test]
fn test_show_unique_messages() {
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

    // Capture the output to test with show_unique=true
    let mut output = Vec::new();
    print_results_to_writer(&result, false, true, &mut output, 5, true).unwrap();

    // Check the output contains the unique messages section
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Unique messages:"));
    assert!(output_str.contains("- NullPointerException"));
    assert!(output_str.contains("- Slow database query"));
    assert!(output_str.contains("- Connection timeout"));
}

#[test]
fn test_top_errors_limit() {
    // Create a mock AnalysisResult with many error types
    let mut result = AnalysisResult {
        matched_lines: vec![],
        count: 10,
        time_trends: HashMap::new(),
        levels_count: HashMap::new(),
        error_types: HashMap::new(),
        unique_messages: HashSet::new(),
    };

    // Add several error types
    result.error_types.insert("Error1".to_string(), 10);
    result.error_types.insert("Error2".to_string(), 9);
    result.error_types.insert("Error3".to_string(), 8);
    result.error_types.insert("Error4".to_string(), 7);
    result.error_types.insert("Error5".to_string(), 6);
    result.error_types.insert("Error6".to_string(), 5);
    result.error_types.insert("Error7".to_string(), 4);
    result.error_types.insert("Error8".to_string(), 3);

    // Test with default limit of 5
    let mut output = Vec::new();
    print_results_to_writer(&result, false, true, &mut output, 5, false).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Error1"));
    assert!(output_str.contains("Error5"));
    assert!(!output_str.contains("Error6")); // Should not include 6th error

    // Test with higher limit
    let mut output = Vec::new();
    print_results_to_writer(&result, false, true, &mut output, 8, false).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Error8")); // Should include 8th error
}
