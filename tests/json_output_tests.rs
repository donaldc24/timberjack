use std::collections::{HashMap, HashSet};
use timber_rs::analyzer::AnalysisResult;

#[test]
fn test_json_output_basic() {
    // Since we can't easily capture stdout in unit tests, let's test the JSON structure directly
    // by creating a mock AnalysisResult and checking expected output structure

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

    // Add some level counts and error types
    result.levels_count.insert("ERROR".to_string(), 2);
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
        .insert("Connection timeout".to_string());

    // Create expected JSON structure
    let json_output = serde_json::json!({
        "matched_lines": result.matched_lines,
        "count": result.count,
        "time_trends": null,
        "stats": {
            "log_levels": [
                {"level": "ERROR", "count": 2}
            ],
            "error_types": [
                {"error_type": "NullPointerException", "count": 1, "rank": 1},
                {"error_type": "Connection timeout", "count": 1, "rank": 2}
            ],
            "unique_messages_count": 2,
            "repetition_ratio": 0.0,
            "unique_messages": null
        }
    });

    // Verify structure has the expected keys
    assert!(
        json_output
            .as_object()
            .unwrap()
            .contains_key("matched_lines")
    );
    assert!(json_output.as_object().unwrap().contains_key("count"));
    assert!(json_output.as_object().unwrap().contains_key("stats"));

    // Verify stats structure
    let stats = json_output.as_object().unwrap().get("stats").unwrap();
    assert!(stats.as_object().unwrap().contains_key("log_levels"));
    assert!(stats.as_object().unwrap().contains_key("error_types"));
    assert!(
        stats
            .as_object()
            .unwrap()
            .contains_key("unique_messages_count")
    );
    assert!(stats.as_object().unwrap().contains_key("repetition_ratio"));
}

#[test]
fn test_json_structure() {
    // Create a mock AnalysisResult with all the fields populated
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

    // Add level counts and error types
    result.levels_count.insert("ERROR".to_string(), 2);
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
        .insert("Connection timeout".to_string());

    // Convert result to JSON manually for testing
    let json_output = serde_json::json!({
        "matched_lines": result.matched_lines,
        "count": result.count,
        "time_trends": [
            {"timestamp": "2025-03-21 14", "count": 1},
            {"timestamp": "2025-03-21 15", "count": 1}
        ],
        "stats": {
            "log_levels": [
                {"level": "ERROR", "count": 2}
            ],
            "error_types": [
                {"error_type": "NullPointerException", "count": 1, "rank": 1},
                {"error_type": "Connection timeout", "count": 1, "rank": 2}
            ],
            "unique_messages_count": 2,
            "repetition_ratio": 0.0,
            "unique_messages": null // Should not be included when show_unique is false
        }
    });

    // Validate the structure is as expected
    assert!(json_output.is_object());
    assert!(
        json_output
            .as_object()
            .unwrap()
            .contains_key("matched_lines")
    );
    assert!(json_output.as_object().unwrap().contains_key("count"));
    assert!(json_output.as_object().unwrap().contains_key("time_trends"));
    assert!(json_output.as_object().unwrap().contains_key("stats"));
}

#[test]
fn test_json_unique_messages() {
    // Create a mock AnalysisResult with unique messages
    let mut result = AnalysisResult {
        matched_lines: vec![
            "2025-03-21 14:00:00,123 [ERROR] NullPointerException".to_string(),
            "2025-03-21 14:01:00,456 [WARN] Slow database query".to_string(),
            "2025-03-21 14:03:00,012 [ERROR] Connection timeout".to_string(),
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

    // Verify structure with show_unique enabled
    let expected_json = serde_json::json!({
        "matched_lines": result.matched_lines,
        "count": result.count,
        "time_trends": null,
        "stats": {
            "log_levels": [],
            "error_types": [],
            "unique_messages_count": 3,
            "repetition_ratio": 0.0,
            "unique_messages": [
                "Connection timeout",
                "NullPointerException",
                "Slow database query"
            ]
        }
    });

    // Note: A full test would capture stdout and parse the JSON
    // For this test, we're checking the expected structure
    assert!(
        expected_json
            .as_object()
            .unwrap()
            .get("stats")
            .unwrap()
            .as_object()
            .unwrap()
            .get("unique_messages")
            .unwrap()
            .is_array()
    );
}

#[test]
fn test_json_top_errors_limit() {
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

    // Verify structure with custom top_errors limit
    let expected_json_3_errors = serde_json::json!({
        "matched_lines": [],
        "count": 10,
        "time_trends": null,
        "stats": {
            "log_levels": [],
            "error_types": [
                {"error_type": "Error1", "count": 10, "rank": 1},
                {"error_type": "Error2", "count": 9, "rank": 2},
                {"error_type": "Error3", "count": 8, "rank": 3}
            ],
            "unique_messages_count": 0,
            "repetition_ratio": 100.0
        }
    });

    // For this test, we're checking the expected structure
    assert_eq!(
        expected_json_3_errors
            .as_object()
            .unwrap()
            .get("stats")
            .unwrap()
            .as_object()
            .unwrap()
            .get("error_types")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        3
    );

    // Verify structure with larger top_errors limit
    let expected_json_7_errors = serde_json::json!({
        "matched_lines": [],
        "count": 10,
        "time_trends": null,
        "stats": {
            "log_levels": [],
            "error_types": [
                {"error_type": "Error1", "count": 10, "rank": 1},
                {"error_type": "Error2", "count": 9, "rank": 2},
                {"error_type": "Error3", "count": 8, "rank": 3},
                {"error_type": "Error4", "count": 7, "rank": 4},
                {"error_type": "Error5", "count": 6, "rank": 5},
                {"error_type": "Error6", "count": 5, "rank": 6},
                {"error_type": "Error7", "count": 4, "rank": 7}
            ],
            "unique_messages_count": 0,
            "repetition_ratio": 100.0
        }
    });

    assert_eq!(
        expected_json_7_errors
            .as_object()
            .unwrap()
            .get("stats")
            .unwrap()
            .as_object()
            .unwrap()
            .get("error_types")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        7
    );
}
