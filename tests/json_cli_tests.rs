use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_cli_json_output() {
    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Some warning").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run timber with JSON output
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd.arg("--json").arg("--stats").arg(file_path).assert();

    // Check that the output looks like valid JSON
    assert
        .success()
        .stdout(predicates::str::contains("{"))
        .stdout(predicates::str::contains("}"))
        .stdout(predicates::str::contains("\"matched_lines\""))
        .stdout(predicates::str::contains("\"count\""))
        .stdout(predicates::str::contains("\"stats\""));
}

#[test]
fn test_cli_top_errors_option() {
    // Create a temporary log file with multiple errors
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:01:00,456 [ERROR] Connection timeout"
    )
    .unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:02:00,789 [ERROR] OutOfMemoryError"
    )
    .unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:03:00,123 [ERROR] FileNotFoundException"
    )
    .unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run timber with custom top_errors limit
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--stats")
        .arg("--top-errors")
        .arg("2")
        .arg(file_path)
        .assert();

    // Should only show the top 2 errors
    assert
        .success()
        .stdout(predicates::str::contains("Top error types:"))
        .stdout(predicates::str::contains("1."))
        .stdout(predicates::str::contains("2."))
        .stdout(predicates::str::contains("3.").not()); // Shouldn't show 3rd error
}

#[test]
fn test_cli_show_unique_option() {
    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Some warning").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run timber with show_unique option
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--stats")
        .arg("--show-unique")
        .arg(file_path)
        .assert();

    // Check that unique messages section appears
    assert
        .success()
        .stdout(predicates::str::contains("Unique messages:"))
        .stdout(predicates::str::contains("NullPointerException")) // Just check for the content
        .stdout(predicates::str::contains("Some warning")); // without assuming formatting
}

#[test]
fn test_cli_json_with_new_options() {
    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Some warning").unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:02:00,789 [ERROR] Connection timeout"
    )
    .unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run timber with JSON output and new options
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--json")
        .arg("--stats")
        .arg("--show-unique")
        .arg("--top-errors")
        .arg("1")
        .arg(file_path)
        .assert();

    // Check that JSON includes unique messages and has limited error types
    assert
        .success()
        .stdout(predicates::str::contains("\"unique_messages\""))
        .stdout(predicates::str::contains("\"error_types\""));

    // In a more comprehensive test, we would parse the JSON and verify:
    // 1. That unique_messages is an array containing the unique messages
    // 2. That error_types contains only one entry (from --top-errors 1)
}
