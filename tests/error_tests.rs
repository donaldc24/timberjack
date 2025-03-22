use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_nonexistent_file() {
    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd
        .arg("--chop")
        .arg("ERROR")
        .arg("nonexistent_file.log")
        .assert();

    // Check for failure but don't check the exact error message
    // This makes the test work across different operating systems
    assert.failure();
}

#[test]
fn test_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd.arg(file_path).assert();

    assert
        .success()
        .stdout(predicates::str::contains("Felled: 0 logs"));
}

#[test]
fn test_malformed_log() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "This is not a properly formatted log line").unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("timber-rs").unwrap();
    let assert = cmd.arg(file_path).assert();

    // Should still process the file, just won't extract much information
    assert
        .success()
        .stdout(predicates::str::contains("Felled: 1 logs"));
}
