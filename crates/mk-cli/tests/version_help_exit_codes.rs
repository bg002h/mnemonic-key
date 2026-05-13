//! v0.3.1 fix: clap soft-error terminations exit 0.
//!
//! Pre-fix, `mk --version` and `mk --help` exited 64 because the
//! `Cli::try_parse()` catch-all in `src/main.rs` mapped EVERY
//! `clap::Error` to `ExitCode::from(64)`. But clap returns soft-error
//! variants for help/version display:
//!
//!   --version → ErrorKind::DisplayVersion
//!   --help    → ErrorKind::DisplayHelp
//!
//! These are not usage errors; the output is to stdout, not stderr, and
//! the canonical Unix convention is exit 0. The fix branches on
//! `e.kind()` and returns `ExitCode::SUCCESS` for those two variants,
//! preserving the catch-all 64 for real parse errors.
//!
//! Discovered during `bg002h/mnemonic-gui` v0.2.0 release prep
//! (companion: `bg002h/mnemonic-gui`).

use assert_cmd::Command;

#[test]
fn version_flag_exits_zero_and_prints_version() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).expect("stdout valid utf-8");
    assert!(
        stdout.starts_with("mk "),
        "expected stdout to start with `mk `; got {stdout:?}"
    );
}

#[test]
fn help_flag_exits_zero_and_prints_help() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).expect("stdout valid utf-8");
    // Clap's --help output includes the `Usage:` header from the
    // top-level `#[command(name = "mk", ...)]` derive.
    assert!(
        stdout.contains("Usage:") && stdout.contains("mk"),
        "expected clap --help stdout to contain `Usage:` and `mk`; got {stdout:?}"
    );
}

#[test]
fn unknown_flag_exits_64() {
    // Regression backstop: the catch-all `_ => ExitCode::from(64)` arm
    // must still fire for real parse errors after the soft-error
    // carve-out above.
    Command::cargo_bin("mk")
        .unwrap()
        .arg("--frob-flag-that-doesnt-exist")
        .assert()
        .failure()
        .code(64);
}
