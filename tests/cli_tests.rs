use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_basic_chop() {
    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Some warning").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run the timber command
    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd.arg("--chop").arg("ERROR").arg(file_path).assert();

    // Check that the output is successful and contains expected text
    assert
        .success()
        .stdout(predicates::str::contains("NullPointerException"))
        .stdout(predicates::str::contains("Felled: 1 logs"));
}

#[test]
fn test_level_filtering() {
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
        "2025-03-21 14:02:00,789 [INFO] Application started"
    )
    .unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run the timber command with level filtering
    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd.arg("--level").arg("WARN").arg(file_path).assert();

    // Check that the output is successful and contains expected text
    assert
        .success()
        .stdout(predicates::str::contains("Some warning"))
        .stdout(predicates::str::contains("NullPointerException").not())
        .stdout(predicates::str::contains("Application started").not())
        .stdout(predicates::str::contains("Felled: 1 logs"));
}

#[test]
fn test_stats_option() {
    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(temp_file, "2025-03-21 14:01:00,456 [WARN] Some warning").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run the timber command with stats
    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd.arg("--stats").arg(file_path).assert();

    // Check that the output contains stats
    assert
        .success()
        .stdout(predicates::str::contains("Stats summary:"))
        .stdout(predicates::str::contains("Log levels:"))
        .stdout(predicates::str::contains("Top error types:"));
}

#[test]
fn test_trend_option() {
    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException"
    )
    .unwrap();
    writeln!(temp_file, "2025-03-21 15:01:00,456 [WARN] Some warning").unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Run the timber command with trend
    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd.arg("--trend").arg(file_path).assert();

    // Check that the output contains trends
    assert
        .success()
        .stdout(predicates::str::contains("Time trends:"))
        .stdout(predicates::str::contains("2025-03-21 14"))
        .stdout(predicates::str::contains("2025-03-21 15"));
}

#[test]
fn test_combined_options() {
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

    // Run the timber command with multiple options
    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg("Exception")
        .arg("--level")
        .arg("ERROR")
        .arg("--stats")
        .arg(file_path)
        .assert();

    // Check that the output combines all options correctly
    assert
        .success()
        .stdout(predicates::str::contains("NullPointerException"))
        .stdout(predicates::str::contains("Connection timeout").not())
        .stdout(predicates::str::contains("Felled: 1 logs"))
        .stdout(predicates::str::contains("Stats summary:"));
}
