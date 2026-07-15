//! Integration tests that exercise the compiled `rshelp` binary directly.
//! Intentionally network-free: they only cover argument parsing, help/
//! version output, and `--clear-cache`, so they run reliably in CI without
//! depending on docs.rs / doc.rust-lang.org being reachable.

use std::process::Command;

fn rshelp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rshelp"))
}

#[test]
fn no_args_prints_help_and_exits_zero() {
    let out = rshelp().output().expect("failed to run rshelp");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("rshelp"), "help output should mention rshelp:\n{stdout}");
}

#[test]
fn help_flag_exits_zero() {
    let out = rshelp().arg("--help").output().expect("failed to run rshelp");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("PATH"), "help output should describe the PATH argument:\n{stdout}");
}

#[test]
fn version_flag_exits_zero() {
    let out = rshelp().arg("--version").output().expect("failed to run rshelp");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.to_lowercase().contains("rshelp"),
        "version output should mention rshelp:\n{stdout}"
    );
}

#[test]
fn clear_cache_exits_zero_with_no_path() {
    let out = rshelp()
        .arg("--clear-cache")
        .arg("--plain")
        .output()
        .expect("failed to run rshelp");
    assert!(out.status.success());
}

#[test]
fn unknown_flag_exits_nonzero() {
    let out = rshelp()
        .arg("--this-flag-does-not-exist")
        .output()
        .expect("failed to run rshelp");
    assert!(!out.status.success());
}
