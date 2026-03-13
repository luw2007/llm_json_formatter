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
fn test_explicit_format_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("test_data.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"));
}

#[test]
fn test_format_with_mode_compact() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
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
        .arg("test_data.json")
        .arg("--mode")
        .arg("pretty")
        .assert()
        .success()
        .stdout(predicate::str::contains("  \"users\": ["));
}

#[test]
fn test_format_with_output_syntax_json5() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("test_data.json")
        .arg("--mode")
        .arg("compact")
        .arg("--output-syntax")
        .arg("json5")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"));
}

#[test]
fn test_shortcut_json5_file() {
    let temp_file = "temp_input.json5";
    fs::write(
        temp_file,
        "{users:[{id:1,name:'Alice'},{id:2,name:'Bob'}],meta:{count:2}}",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("Alice"));

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_shortcut_file_without_extension() {
    let temp_file = "temp_input_no_ext";
    fs::write(temp_file, r#"{"users":[{"id":1,"name":"Alice"}]}"#).unwrap();

    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("Alice"));

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_analyze_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("analyze")
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
        .arg("test_data.json")
        .arg("-p")
        .arg("users[0].name")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn test_search_key_fuzzy_command() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("search")
        .arg("test_data.json")
        .arg("--key")
        .arg("na")
        .assert()
        .success()
        .stdout(predicate::str::contains("users[0].name"))
        .stdout(predicate::str::contains("users[1].name"));
}

#[test]
fn test_invalid_json_file() {
    let temp_file = "temp_invalid.json";
    fs::write(temp_file, "{invalid json}").unwrap();

    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg(temp_file).assert().failure();

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
        .stdout(predicate::str::contains("Default format"))
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
    let temp_file = "temp_key_order.json";
    fs::write(temp_file, r#"{"z":1,"a":2,"m":3}"#).unwrap();

    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{"a":2,"m":3,"z":1\}"#).unwrap());

    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_format_with_entities_option() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .arg("test_data.json")
        .arg("--entities")
        .arg("users[*]")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"));
}

#[test]
fn test_format_requires_input_file() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("<INPUT>"));
}

#[test]
fn test_pipe_shortcut_without_args() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.write_stdin(r#"{"users":[{"id":1,"name":"Alice"}]}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn test_pipe_shortcut_with_double_dash() {
    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("--")
        .write_stdin(r#"{"users":[{"id":2,"name":"Bob"}]}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn test_prompt_uses_wildcard_for_object_map_paths() {
    let temp_file = "temp_prompt_map.json";
    fs::write(temp_file, r#"{"by_id":{"u1":{"name":"Alice"},"u2":{"age":30}}}"#).unwrap();

    let mut cmd = Command::cargo_bin("jf").unwrap();
    cmd.arg("prompt")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Path: by_id[*]"));

    fs::remove_file(temp_file).unwrap();
}
