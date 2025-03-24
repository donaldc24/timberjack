use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_json_field_filtering() {
    // Create a temporary log file with JSON logs
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"{{"timestamp":"2025-03-21T14:00:00.123Z","level":"ERROR","service":"api","user_id":"12345","message":"Database connection failed"}}"#
    )
        .unwrap();
    writeln!(
        temp_file,
        r#"{{"timestamp":"2025-03-21T14:01:00.456Z","level":"WARN","service":"web","user_id":"67890","message":"Slow query detected"}}"#
    )
        .unwrap();
    writeln!(
        temp_file,
        r#"{{"timestamp":"2025-03-21T14:02:00.789Z","level":"ERROR","service":"auth","user_id":"12345","message":"Authentication failed"}}"#
    )
        .unwrap();

    let file_path = temp_file.path().to_str().unwrap();

    // Test filtering by service=api
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--format")
        .arg("json")
        .arg("-f")
        .arg("service=api")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("Database connection failed"))
        .stdout(predicates::str::contains("Slow query detected").not())
        .stdout(predicates::str::contains("Authentication failed").not())
        .stdout(predicates::str::contains("Felled: 1 logs"));

    // Test filtering by user_id=12345
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--format")
        .arg("json")
        .arg("-f")
        .arg("user_id=12345")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("Database connection failed"))
        .stdout(predicates::str::contains("Slow query detected").not())
        .stdout(predicates::str::contains("Authentication failed"))
        .stdout(predicates::str::contains("Felled: 2 logs"));

    // Test filtering by multiple fields
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--format")
        .arg("json")
        .arg("-f")
        .arg("level=ERROR")
        .arg("-f")
        .arg("service=auth")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("Database connection failed").not())
        .stdout(predicates::str::contains("Slow query detected").not())
        .stdout(predicates::str::contains("Authentication failed"))
        .stdout(predicates::str::contains("Felled: 1 logs"));

    // Test combining field filtering with pattern filtering
    let mut cmd = Command::cargo_bin("timber").unwrap();
    let assert = cmd
        .arg("--format")
        .arg("json")
        .arg("--chop")
        .arg("failed")
        .arg("-f")
        .arg("user_id=12345")
        .arg(file_path)
        .assert();

    assert
        .success()
        .stdout(predicates::str::contains("Database connection failed"))
        .stdout(predicates::str::contains("Authentication failed"))
        .stdout(predicates::str::contains("Slow query detected").not())
        .stdout(predicates::str::contains("Felled: 2 logs"));
}
