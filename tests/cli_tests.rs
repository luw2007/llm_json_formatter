use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_shortcut_single_file() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("test_data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn test_shortcut_multiple_files() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("test_data.json")
        .arg("test_map.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("users_by_id"));
}

#[test]
fn test_shortcut_pipe_input() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.write_stdin(r#"{"test":"value","number":123}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("number"));
}

#[test]
fn test_explicit_format_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("-i")
        .arg("test_data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"));
}

#[test]
fn test_format_with_mode_compact() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("-i")
        .arg("test_data.json")
        .arg("--mode")
        .arg("compact")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{"meta":\{"count":2,"version":"1.0"\}.*\}"#).unwrap());
}

#[test]
fn test_format_with_mode_pretty() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("-i")
        .arg("test_data.json")
        .arg("--mode")
        .arg("pretty")
        .assert()
        .success()
        .stdout(predicate::str::contains("  \"users\": ["));
}

#[test]
fn test_pipe_with_compact_mode() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.write_stdin(r#"{"test":"value"}"#)
        .arg("format")
        .arg("--mode")
        .arg("compact")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{"test":"value"\}"#).unwrap());
}

#[test]
fn test_analyze_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("analyze")
        .arg("-i")
        .arg("test_data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("JSON Analysis:"))
        .stdout(predicate::str::contains("Byte Size:"))
        .stdout(predicate::str::contains("Max Depth:"));
}

#[test]
fn test_schema_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("schema")
        .arg("-i")
        .arg("test_data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("number"))
        .stdout(predicate::str::contains("string"));
}

#[test]
fn test_paths_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("paths")
        .arg("-i")
        .arg("test_data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("meta"));
}

#[test]
fn test_search_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("search")
        .arg("-i")
        .arg("test_data.json")
        .arg("-p")
        .arg("users[0].name")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn test_invalid_json_file() {
    let temp_file = "temp_invalid.json";
    fs::write(temp_file, "{invalid json}").unwrap();

    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg(temp_file)
        .assert()
        .failure();

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_nonexistent_file() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input"));
}

#[test]
fn test_help_displays_examples() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Quick format"))
        .stdout(predicate::str::contains("Pipe input shortcut"))
        .stdout(predicate::str::contains("jf data.json"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("jf"));
}

#[test]
fn test_shortcut_preserves_key_order_alphabetic() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.write_stdin(r#"{"z":1,"a":2,"m":3}"#)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{"a":2,"m":3,"z":1\}"#).unwrap());
}

#[test]
fn test_format_with_entities_option() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("-i")
        .arg("test_data.json")
        .arg("--entities")
        .arg("users[*]")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"));
}
