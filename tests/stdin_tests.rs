use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_stdin_basic() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Write to stdin using the provided API
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 14:01:00,456 [WARN] Some warning\n",
    )
    .unwrap();

    // Execute command
    let assert = cmd.assert();

    // Check output
    assert
        .success()
        .stdout(predicate::str::contains("NullPointerException"))
        .stdout(predicate::str::contains("Some warning"))
        .stdout(predicate::str::contains("Felled: 2 logs"));
}

#[test]
fn test_stdin_with_pattern() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command with chop option
    cmd.arg("--chop").arg("ERROR");

    // Write to stdin
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 14:01:00,456 [WARN] Some warning\n",
    )
    .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("NullPointerException"))
        .stdout(predicate::str::contains("Some warning").not())
        .stdout(predicate::str::contains("Felled: 1 logs"));
}

#[test]
fn test_stdin_with_level() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command with level filter
    cmd.arg("--level").arg("WARN");

    // Write to stdin
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 14:01:00,456 [WARN] Some warning\n\
         2025-03-21 14:02:00,789 [INFO] Application started\n",
    )
    .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("Some warning"))
        .stdout(predicate::str::contains("NullPointerException").not())
        .stdout(predicate::str::contains("Application started").not())
        .stdout(predicate::str::contains("Felled: 1 logs"));
}

#[test]
fn test_stdin_with_stats() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command with stats option
    cmd.arg("--stats");

    // Write to stdin
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 14:01:00,456 [WARN] Some warning\n",
    )
    .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("Stats summary:"))
        .stdout(predicate::str::contains("Log levels:"))
        .stdout(predicate::str::contains("ERROR: 1 log"))
        .stdout(predicate::str::contains("WARN: 1 log"));
}

#[test]
fn test_stdin_with_trends() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command with trend option
    cmd.arg("--trend");

    // Write to stdin
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 15:01:00,456 [WARN] Some warning\n",
    )
    .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("Time trends:"))
        .stdout(predicate::str::contains("2025-03-21 14"))
        .stdout(predicate::str::contains("2025-03-21 15"));
}

#[test]
fn test_stdin_json_output() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command for JSON output
    cmd.arg("--json");

    // Write to stdin
    cmd.write_stdin("2025-03-21 14:00:00,123 [ERROR] NullPointerException\n")
        .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("}"))
        .stdout(predicate::str::contains("NullPointerException"));
}

#[test]
fn test_stdin_combined_options() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command with multiple options
    cmd.arg("--chop")
        .arg("ERROR")
        .arg("--level")
        .arg("ERROR")
        .arg("--stats");

    // Write to stdin
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 14:01:00,456 [WARN] Some warning\n\
         2025-03-21 14:02:00,789 [ERROR] Connection timeout\n",
    )
    .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("NullPointerException"))
        .stdout(predicate::str::contains("Connection timeout"))
        .stdout(predicate::str::contains("Some warning").not())
        .stdout(predicate::str::contains("Stats summary:"))
        .stdout(predicate::str::contains("ERROR: 2 logs"));
}

#[test]
fn test_stdin_empty() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Write empty stdin (just create the pipe)
    cmd.write_stdin("").unwrap();

    // Execute and verify
    let assert = cmd.assert();

    assert
        .success()
        .stdout(predicate::str::contains("Felled: 0 logs"));
}

#[test]
fn test_stdin_with_count() {
    let mut cmd = Command::cargo_bin("timber").unwrap();

    // Set up command with count option
    cmd.arg("--count");

    // Write to stdin
    cmd.write_stdin(
        "2025-03-21 14:00:00,123 [ERROR] NullPointerException\n\
         2025-03-21 14:01:00,456 [WARN] Some warning\n",
    )
    .unwrap();

    // Execute and verify
    let assert = cmd.assert();

    // For count, we expect only the number as output
    assert.success().stdout(predicate::str::contains("2"));
}
