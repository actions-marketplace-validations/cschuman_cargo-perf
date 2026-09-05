//! CLI integration tests for cargo-perf binary.
//!
//! Tests the command-line interface behavior.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a Command for the cargo-perf binary.
fn cargo_perf() -> Command {
    cargo_bin_cmd!("cargo-perf")
}

#[test]
fn test_help_flag() {
    cargo_perf()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preventive performance analysis"));
}

#[test]
fn test_version_flag() {
    cargo_perf()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo-perf"));
}

#[test]
fn test_rules_subcommand() {
    cargo_perf()
        .arg("rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("async-block-in-async"))
        .stdout(predicate::str::contains("lock-across-await"))
        .stdout(predicate::str::contains("clone-in-hot-loop"))
        .stdout(predicate::str::contains("n-plus-one-query"));
}

#[test]
fn test_explain_known_rule() {
    cargo_perf()
        .arg("explain")
        .arg("async-block-in-async")
        .assert()
        .success()
        .stdout(predicate::str::contains("Why it matters"))
        .stdout(predicate::str::contains("Blocking calls in async"));
}

#[test]
fn test_explain_unknown_rule() {
    cargo_perf()
        .arg("explain")
        .arg("nonexistent-rule")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown rule"));
}

#[test]
fn test_init_creates_config() {
    let temp = TempDir::new().unwrap();

    cargo_perf()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    assert!(temp.path().join("cargo-perf.toml").exists());
}

#[test]
fn test_init_fails_if_exists() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("cargo-perf.toml"), "").unwrap();

    cargo_perf()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_check_clean_code() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("clean.rs"),
        r#"
fn main() {
    let x = 1 + 2;
    println!("{}", x);
}
"#,
    )
    .unwrap();

    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .assert()
        .success();
}

#[test]
fn test_check_finds_issues() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("async-block-in-async"));
}

#[test]
fn test_check_json_output() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // --format is a global option, must come before subcommand
    cargo_perf()
        .arg("--format")
        .arg("json")
        .arg("--path")
        .arg(temp.path())
        .assert()
        .success()
        // JSON output is pretty-printed
        .stdout(predicate::str::contains(
            r#""rule_id": "async-block-in-async""#,
        ));
}

#[test]
fn test_check_sarif_output() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // --format is a global option, must come before subcommand
    cargo_perf()
        .arg("--format")
        .arg("sarif")
        .arg("--path")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("$schema"))
        .stdout(predicate::str::contains("sarif-schema"))
        .stdout(predicate::str::contains("ruleId"));
}

#[test]
fn test_check_strict_mode() {
    let temp = TempDir::new().unwrap();
    // Clone in loop is NOT a strict rule
    fs::write(
        temp.path().join("code.rs"),
        r#"
fn test(data: &[String]) {
    for s in data {
        let _ = s.clone();
    }
}
"#,
    )
    .unwrap();

    // Without --strict, should report clone-in-hot-loop
    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("clone-in-hot-loop"));

    // With --strict, should NOT report clone-in-hot-loop
    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .arg("--strict")
        .assert()
        .success()
        .stdout(predicate::str::contains("clone-in-hot-loop").not());
}

#[test]
fn test_check_timing_flag() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .arg("--timing")
        .assert()
        .success()
        .stderr(predicate::str::contains("Analysis time"));
}

#[test]
fn test_check_fail_on_error() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // async-block-in-async is Error severity, so --fail-on=error should fail
    // --fail-on is a global option
    cargo_perf()
        .arg("--fail-on")
        .arg("error")
        .arg("--path")
        .arg(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("diagnostic(s) at or above"));
}

#[test]
fn test_check_nonexistent_path() {
    // Using --path with nonexistent path should fail during config load
    cargo_perf()
        .arg("--path")
        .arg("/nonexistent/path/to/project")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_baseline_creates_file() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("code.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // Create baseline
    cargo_perf()
        .arg("baseline")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Baseline created"));

    // Baseline file should exist
    assert!(temp.path().join(".cargo-perf-baseline").exists());

    // Read the baseline file and verify it contains our rule
    let baseline_content = fs::read_to_string(temp.path().join(".cargo-perf-baseline")).unwrap();
    assert!(baseline_content.contains("async-block-in-async"));
}

#[test]
fn test_check_with_baseline_flag() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

    // Check with --baseline when no baseline file exists should warn
    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .arg("--baseline")
        .assert()
        .success()
        .stderr(predicate::str::contains("No baseline file found"));
}

#[test]
fn test_fix_dry_run() {
    let temp = TempDir::new().unwrap();
    // Exercise the dry-run PREVIEW path, which is only reached when a diagnostic
    // is fixable. A blocking `std::thread::sleep` in an async fn still carries an
    // autofix (-> `tokio::time::sleep(..).await`). NOTE: the collect-then-iterate
    // autofix this test used to rely on was intentionally removed (D18) because
    // splicing out `.collect().iter()` can change the resulting type/borrow and
    // produce non-compiling code, so that case no longer reaches this branch.
    let code = r#"
async fn slow() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#;
    fs::write(temp.path().join("fix.rs"), code).unwrap();

    cargo_perf()
        .arg("fix")
        .arg(temp.path())
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    // Dry run must leave the file byte-for-byte unchanged.
    let after = fs::read_to_string(temp.path().join("fix.rs")).unwrap();
    assert_eq!(code, after);
}

#[test]
fn test_default_command_is_check() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // Running without subcommand should do check
    cargo_perf()
        .arg("--path")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("async-block-in-async"));
}

// Cargo runs external subcommands as `cargo-perf perf <args>`: the subcommand
// name arrives as argv[1]. Every documented entry point (`cargo perf ...`)
// goes through this path, so these tests pass "perf" exactly as cargo does.

#[test]
fn test_cargo_subcommand_invocation_version() {
    cargo_perf()
        .arg("perf")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("cargo-perf "))
        .stdout(predicate::str::is_match(r"^cargo-perf \d+\.\d+\.\d+").unwrap());
}

#[test]
fn test_cargo_subcommand_invocation_rules() {
    cargo_perf()
        .arg("perf")
        .arg("rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("async-block-in-async"));
}

#[test]
fn test_cargo_subcommand_invocation_check() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    cargo_perf()
        .arg("perf")
        .arg("check")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("async-block-in-async"));
}

#[test]
fn test_cargo_subcommand_invocation_default_check() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // `cargo perf` with no subcommand runs check on --path
    cargo_perf()
        .arg("perf")
        .arg("--path")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("async-block-in-async"));
}

// action.yml runs `cargo perf check "$PATH" --format sarif` and the getting
// started guide documents `cargo perf check --baseline --fail-on error`:
// output and threshold flags must be accepted after the subcommand too.

#[test]
fn test_check_format_flag_after_subcommand() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // Exact shape used by action.yml
    cargo_perf()
        .arg("perf")
        .arg("check")
        .arg(temp.path())
        .arg("--format")
        .arg("sarif")
        .assert()
        .success()
        .stdout(predicate::str::contains("sarif-schema"))
        .stdout(predicate::str::contains("ruleId"));
}

#[test]
fn test_check_fail_on_flag_after_subcommand() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("bad.rs"),
        r#"
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
    )
    .unwrap();

    // async-block-in-async is Error severity, so this must fail
    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .arg("--fail-on")
        .arg("error")
        .assert()
        .failure()
        .stderr(predicate::str::contains("diagnostic(s) at or above"));
}

#[test]
fn test_check_min_severity_flag_after_subcommand() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("code.rs"),
        r#"
fn test(data: &[String]) {
    for s in data {
        let _ = s.clone();
    }
}
"#,
    )
    .unwrap();

    // Positive control: the rule fires on this fixture with no floor set.
    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("clone-in-hot-loop"));

    // clone-in-hot-loop is below Error, so raising the floor hides it
    cargo_perf()
        .arg("check")
        .arg(temp.path())
        .arg("--min-severity")
        .arg("error")
        .assert()
        .success()
        .stdout(predicate::str::contains("clone-in-hot-loop").not());
}
