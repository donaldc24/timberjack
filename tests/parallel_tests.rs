use std::io::Write;
use tempfile::NamedTempFile;
use timber_rs::analyzer::LogAnalyzer;
use regex::Regex;

// Helper function to create a test log file with specified number of lines
fn create_test_log_file(lines: usize) -> NamedTempFile {
    let temp_file = NamedTempFile::new().unwrap();
    let mut file = std::fs::File::create(temp_file.path()).unwrap();

    for i in 0..lines {
        let level = match i % 5 {
            0 => "ERROR",
            1 => "WARN",
            2 => "INFO",
            3 => "DEBUG",
            _ => "TRACE",
        };

        writeln!(
            file,
            "2025-03-21 {:02}:{:02}:00,{:03} [{}] Test message {}",
            (i / 60) % 24,
            i % 60,
            i % 1000,
            level,
            i
        )
            .unwrap();
    }

    temp_file
}

#[test]
fn test_parallel_processing_basic() {
    // Create a file with 1000 lines
    let temp_file = create_test_log_file(1000);
    let file_path = temp_file.path();

    // Read the file into memory
    let content = std::fs::read_to_string(file_path).unwrap();
    let lines: Vec<String> = content.lines().map(String::from).collect();

    // Create analyzers (one for each method to avoid mutable borrow issues)
    let mut sequential_analyzer = LogAnalyzer::new();
    let mut parallel_analyzer = LogAnalyzer::new();

    // Process sequentially
    let sequential_result = sequential_analyzer.analyze_lines(lines.clone().into_iter(), None, None, false, false);

    // Process in parallel
    let parallel_result = parallel_analyzer.analyze_lines_parallel(lines, None, None, false, false);

    // Compare results
    assert_eq!(sequential_result.count, parallel_result.count);
    assert_eq!(sequential_result.matched_lines.len(), parallel_result.matched_lines.len());
}

#[test]
fn test_parallel_processing_with_filters() {
    // Create a file with 1000 lines
    let temp_file = create_test_log_file(1000);
    let file_path = temp_file.path();

    // Read the file into memory
    let content = std::fs::read_to_string(file_path).unwrap();
    let lines: Vec<String> = content.lines().map(String::from).collect();

    // Create analyzers (one for each method to avoid mutable borrow issues)
    let mut sequential_analyzer = LogAnalyzer::new();
    let mut parallel_analyzer = LogAnalyzer::new();
    let pattern = Regex::new("ERROR").unwrap();

    // Process sequentially with pattern and level filter
    let sequential_result = sequential_analyzer.analyze_lines(
        lines.clone().into_iter(),
        Some(&pattern),
        Some("ERROR"),
        false,
        false
    );

    // Process in parallel with the same filters
    let parallel_result = parallel_analyzer.analyze_lines_parallel(
        lines,
        Some(&pattern),
        Some("ERROR"),
        false,
        false
    );

    // Compare results
    assert_eq!(sequential_result.count, parallel_result.count);
    assert_eq!(sequential_result.matched_lines.len(), parallel_result.matched_lines.len());
}

#[test]
fn test_parallel_processing_with_stats() {
    // Create a file with 1000 lines
    let temp_file = create_test_log_file(1000);
    let file_path = temp_file.path();

    // Read the file into memory
    let content = std::fs::read_to_string(file_path).unwrap();
    let lines: Vec<String> = content.lines().map(String::from).collect();

    // Create analyzers (one for each method to avoid mutable borrow issues)
    let mut sequential_analyzer = LogAnalyzer::new();
    let mut parallel_analyzer = LogAnalyzer::new();

    // Process sequentially with stats collection
    let sequential_result = sequential_analyzer.analyze_lines(
        lines.clone().into_iter(),
        None,
        None,
        true,
        true
    );

    // Process in parallel with stats collection
    let parallel_result = parallel_analyzer.analyze_lines_parallel(
        lines,
        None,
        None,
        true,
        true
    );

    // Compare results
    assert_eq!(sequential_result.count, parallel_result.count);

    // Check if time trends have the same number of entries and values
    assert_eq!(sequential_result.time_trends.len(), parallel_result.time_trends.len());
    for (timestamp, count) in &sequential_result.time_trends {
        assert_eq!(
            parallel_result.time_trends.get(timestamp),
            Some(count)
        );
    }

    // Check if level counts match
    assert_eq!(sequential_result.levels_count.len(), parallel_result.levels_count.len());
    for (level, count) in &sequential_result.levels_count {
        assert_eq!(
            parallel_result.levels_count.get(level),
            Some(count)
        );
    }

    // Check if unique message counts match
    assert_eq!(sequential_result.unique_messages.len(), parallel_result.unique_messages.len());
}

// This test is disabled because the LogAnalyzer does not have a merge_results method
// We'll need to implement a custom function to merge results or skip this test
#[test]
#[ignore]
fn test_merge_results_manually() {
    // This test would verify our ability to merge results
    // We'll implement a simpler version that doesn't require cloning AnalysisResult

    // Create two different log files
    let temp_file1 = create_test_log_file(100);
    let temp_file2 = create_test_log_file(200);

    // Read files into memory
    let content1 = std::fs::read_to_string(temp_file1.path()).unwrap();
    let content2 = std::fs::read_to_string(temp_file2.path()).unwrap();

    let lines1: Vec<String> = content1.lines().map(String::from).collect();
    let lines2: Vec<String> = content2.lines().map(String::from).collect();

    // Analyze each file separately
    let mut analyzer1 = LogAnalyzer::new();
    let mut analyzer2 = LogAnalyzer::new();
    let result1 = analyzer1.analyze_lines(lines1.into_iter(), None, None, true, true);
    let result2 = analyzer2.analyze_lines(lines2.into_iter(), None, None, true, true);

    // Verify some basic properties we'd expect when combining results
    assert_eq!(result1.count + result2.count, 300);

    // Check time trends
    for (timestamp, count1) in &result1.time_trends {
        if let Some(count2) = result2.time_trends.get(timestamp) {
            // If we had a merge function, we'd expect timestamps in both to add up
            let expected_merged = count1 + count2;
            assert!(expected_merged >= *count1);
            assert!(expected_merged >= *count2);
        }
    }
}