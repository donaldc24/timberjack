use assert_cmd::Command;
use std::path::Path;

// Helper function to check if a sample log file exists
fn sample_log_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// Tests for small log file
#[test]
fn test_small_log_pattern_search() {
    let log_path = "tests/test_logs/sml/app-0.log";
    if !sample_log_exists(log_path) {
        println!("Skipping test - sample log not found: {}", log_path);
        return;
    }

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg("ERROR")
        .arg(log_path)
        .assert();

    assert.success();
}

#[test]
fn test_small_log_stats() {
    let log_path = "tests/test_logs/sml/app-0.log";
    if !sample_log_exists(log_path) {
        println!("Skipping test - sample log not found: {}", log_path);
        return;
    }

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--stats")
        .arg(log_path)
        .assert();

    assert.success()
        .stdout(predicates::str::contains("Stats summary:"));
}

// Tests for medium log file
#[test]
fn test_medium_log_level_filtering() {
    let log_path = "tests/test_logs/med/web_server-0.log";
    if !sample_log_exists(log_path) {
        println!("Skipping test - sample log not found: {}", log_path);
        return;
    }

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--level")
        .arg("ERROR")
        .arg(log_path)
        .assert();

    assert.success();
}

// Tests for large log file
#[test]
fn test_large_log_trend_analysis() {
    let log_path = "tests/test_logs/lrg/app_errors-0.log";
    if !sample_log_exists(log_path) {
        println!("Skipping test - sample log not found: {}", log_path);
        return;
    }

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--trend")
        .arg(log_path)
        .assert();

    assert.success()
        .stdout(predicates::str::contains("Time trends:"));
}

#[test]
fn test_large_log_error_spike_detection() {
    let log_path = "tests/test_logs/lrg/app_errors-0.log";
    if !sample_log_exists(log_path) {
        println!("Skipping test - sample log not found: {}", log_path);
        return;
    }

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--level")
        .arg("ERROR")
        .arg("--trend")
        .arg(log_path)
        .assert();

    // We know from previous runs that there's an error spike at 4:00 AM
    assert.success()
        .stdout(predicates::str::contains("2025-03-21 04"))
        .stdout(predicates::str::contains("logs occurred during this hour"));
}

#[test]
fn test_combined_features() {
    let log_path = "tests/test_logs/lrg/app_errors-0.log";
    if !sample_log_exists(log_path) {
        println!("Skipping test - sample log not found: {}", log_path);
        return;
    }

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg("Connection")
        .arg("--level")
        .arg("ERROR")
        .arg("--trend")
        .arg("--stats")
        .arg(log_path)
        .assert();

    assert.success()
        .stdout(predicates::str::contains("Time trends:"))
        .stdout(predicates::str::contains("Stats summary:"))
        .stdout(predicates::str::contains("Connection timeout"));
}