use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn new_creates_project_layout() {
    let temp = TempDir::new().unwrap();
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jalmt"));
    cmd.arg("new").arg("demo").arg("--dir").arg(temp.path());
    cmd.assert().success();

    let root = temp.path().join("demo");
    assert!(root.join("jalm.toml").exists());
    assert!(root.join("jalm.lock").exists());
    assert!(root.join("src/main.jalm").exists());
    assert!(root.join("tests/basic.jalm").exists());
}

#[test]
fn check_reports_diagnostics_json() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("main.jalm");
    fs::write(&file, "fn main() -> i64 { return 0; }").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("jalmt"));
    cmd.arg("check").arg(&file);
    cmd.assert().success().stdout(predicate::str::contains("type_diagnostics"));
}
