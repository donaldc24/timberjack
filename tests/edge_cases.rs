// tests/edge_cases.rs
use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd.arg(file_path).assert();

    assert
        .success()
        .stdout(predicates::str::contains("Felled: 0 logs"));
}

#[test]
fn test_malformed_logs() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "This is not a properly formatted log line").unwrap();
    writeln!(temp_file, "Another invalid log line without timestamp or level").unwrap();
    writeln!(temp_file, "2025-03-21 Some malformed timestamp [INFO] Message").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd.arg(file_path).assert();

    // Should still process the file, just won't extract much information
    assert
        .success()
        .stdout(predicates::str::contains("Felled: 3 logs")); // All lines should be included
}

#[test]
fn test_non_ascii_characters() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "2025-03-21 14:00:00,123 [ERROR] Résumé upload failed").unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Ümlaut encoding issue").unwrap();
    writeln!(temp_file, "2025-03-21 14:02:00,789 [INFO] 你好，世界! (Hello, world!)").unwrap();
    writeln!(temp_file, "2025-03-21 14:03:00,012 [ERROR] エラーが発生しました (Error occurred)").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd.arg(file_path).assert();

    assert
        .success()
        .stdout(predicates::str::contains("Résumé"))
        .stdout(predicates::str::contains("Ümlaut"))
        .stdout(predicates::str::contains("你好，世界!"))
        .stdout(predicates::str::contains("エラーが発生しました"));
}

#[test]
fn test_very_large_values() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "2025-03-21 14:00:00,123 [ERROR] Error 1").unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [ERROR] Error 2").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Test with an extremely large --top-errors value
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--stats")
        .arg("--top-errors")
        .arg("1000000")
        .arg(file_path)
        .assert();

    // Check if the output contains error types, without assuming specific formatting
    assert
        .success()
        .stdout(predicates::str::contains("Error 1"))
        .stdout(predicates::str::contains("Error 2"));
}

#[test]
fn test_complex_regex_patterns() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "2025-03-21 14:00:00,123 [ERROR] NullPointerException").unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Connection timeout").unwrap();
    writeln!(temp_file, "2025-03-21 14:02:00,789 [INFO] User123 logged in").unwrap();
    writeln!(temp_file, "2025-03-21 14:03:00,012 [ERROR] Invalid user ID: ABC-123-XYZ").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Test with complex regex pattern
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg(r"(?i)^.*\[(?:ERROR|WARN)\].*(?:Exception|timeout).*$")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("NullPointerException"))
        .stdout(predicates::str::contains("Connection timeout"))
        .stdout(predicates::str::contains("User123 logged in").not())
        .stdout(predicates::str::contains("Invalid user ID").not());

    // Test with lookahead/lookbehind regex
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg(r"ID:\s+\w+-\d+-\w+")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("Invalid user ID: ABC-123-XYZ"))
        .stdout(predicates::str::contains("NullPointerException").not());
}

#[test]
fn test_mixed_log_formats() {
    let mut temp_file = NamedTempFile::new().unwrap();
    // Standard format
    writeln!(temp_file, "2025-03-21 14:00:00,123 [ERROR] NullPointerException").unwrap();
    // Apache-like format - note this won't be detected as ERROR by current implementation
    writeln!(temp_file, "127.0.0.1 - - [21/Mar/2025:14:01:00 +0000] \"GET /index.html HTTP/1.1\" 200 1234").unwrap();
    // Simple format - note this won't be detected as ERROR by current implementation
    writeln!(temp_file, "ERROR: Database connection failed at 14:02:00").unwrap();
    // JSON-like format - note this won't be detected as ERROR by current implementation
    writeln!(temp_file, "{{\"timestamp\":\"2025-03-21T14:03:00.123Z\",\"level\":\"ERROR\",\"message\":\"Failed login attempt\"}}").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Should handle mixed formats and find the standard format ERROR
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--level")
        .arg("ERROR")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("NullPointerException"));

    // Test without level filter to see all lines
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("NullPointerException"))
        .stdout(predicates::str::contains("GET /index.html"))
        .stdout(predicates::str::contains("ERROR: Database connection"))
        .stdout(predicates::str::contains("Failed login attempt"));
}

#[test]
fn test_very_long_lines() {
    let mut temp_file = NamedTempFile::new().unwrap();
    // Create a very long log message (10KB+)
    let long_message = format!("2025-03-21 14:00:00,123 [ERROR] Very long error message: {}", "A".repeat(10_000));
    writeln!(temp_file, "{}", long_message).unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd.arg(file_path).assert();

    // Should handle very long lines gracefully
    assert
        .success()
        .stdout(predicates::str::contains("Very long error message"));
}

#[test]
fn test_multiple_matches_same_line() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "2025-03-21 14:00:00,123 [ERROR] Error Error Error multiple matches in one line").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg("Error")
        .arg(file_path)
        .assert();

    // Should only count the line once despite multiple matches
    assert
        .success()
        .stdout(predicates::str::contains("Error Error Error"))
        .stdout(predicates::str::contains("Felled: 1 logs"));
}