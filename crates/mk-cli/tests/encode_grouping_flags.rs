//! `mk encode` display-grouping flags (single print, Brian 2026-11-22).
//!
//! Each mk1 string is printed ONCE, on stdout. `--group-size N` groups stdout
//! on request (default 0, the canonical unbroken artifact), as `mt encode` and
//! `mnemonic bundle` do; there is no stderr engraving card. The stdout/stderr
//! byte rows live in `single_print.rs`; this file keeps the flag-surface rows.
//!
//! `--json` is UNCHANGED: it ignores the grouping flags.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";

/// The one advisory line `mk encode` has always ended its stderr with. The card
/// is built in FRONT of it, never in place of it.
const ADVISORY: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend";

fn encode(extra: &[&str]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.args([
        "encode",
        "--xpub",
        V1_XPUB,
        "--origin-fingerprint",
        "aabbccdd",
        "--origin-path",
        "m/48'/0'/0'/2'",
        "--policy-id-stub",
        "11223344",
    ]);
    cmd.args(extra);
    cmd.output().unwrap()
}

fn stdout_of(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn stderr_of(out: &std::process::Output) -> String {
    String::from_utf8(out.stderr.clone()).unwrap()
}

fn first_line(out: &std::process::Output) -> String {
    stdout_of(out).lines().next().unwrap().to_string()
}

/// Insert `sep` every `n` chars — the reference rendering, recomputed here
/// rather than imported, so the test measures the binary instead of agreeing
/// with it.
fn regroup(s: &str, n: usize, sep: char) -> String {
    if n == 0 {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % n == 0 {
            out.push(sep);
        }
        out.push(c);
    }
    out
}

// ──────────────────────────────────────────────────────────────────────────
// §6a — stdout is the artifact and nothing else.
// ──────────────────────────────────────────────────────────────────────────

/// Every stdout line is an UNBROKEN mk1 string, under the DEFAULT flags.
///
/// The default is what a pipeline gets, so the default is what the rule is
/// about: `mk encode | me sysw pack` fails on a grouped card and there is no
/// flag in that pipeline to fix it.
#[test]
fn encode_stdout_is_the_unbroken_artifact_under_default_flags() {
    let out = encode(&[]);
    assert!(out.status.success(), "stderr={}", stderr_of(&out));
    let stdout = stdout_of(&out);
    assert!(!stdout.is_empty(), "expected at least one mk1 line");
    for line in stdout.lines() {
        assert!(
            line.starts_with("mk1"),
            "every stdout line is an mk1 string; got {line:?}"
        );
        assert!(
            !line.contains(' ') && !line.contains('-') && !line.contains(','),
            "stdout carries no display separator; got {line:?}"
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────
// `--json` ignores the grouping flags.
// ──────────────────────────────────────────────────────────────────────────

/// `--json` is unchanged: the JSON envelope is the ONLY thing on stdout, and
/// stderr carries only the advisory.
#[test]
fn json_mode_emits_no_card() {
    let out = encode(&["--json"]);
    assert!(out.status.success(), "stderr={}", stderr_of(&out));
    let stdout = stdout_of(&out);
    assert_eq!(stdout.lines().count(), 1, "one JSON line; got {stdout:?}");
    assert!(stdout.starts_with('{'));
    let stderr = stderr_of(&out);
    assert_eq!(
        stderr.trim_end(),
        ADVISORY,
        "no card in --json mode; stderr={stderr:?}"
    );
}

/// An unknown separator is an exit-64 parse error (unchanged by this entry).
#[test]
fn encode_rejects_bad_separator() {
    let out = encode(&["--separator", "bogus"]);
    assert_eq!(out.status.code(), Some(64), "bad separator → exit 64");
}
// ──────────────────────────────────────────────────────────────────────────
// §6c — the separator narrows to whitespace.
// ──────────────────────────────────────────────────────────────────────────

/// `hyphen` and `comma` are gone, and the refusal says what to use instead
/// (§6h: remedy text must be executable).
///
/// Exit 64: `mk` maps every clap parse error to 64 (`main.rs`), and
/// `--separator` is a clap `value_parser`.
#[test]
fn retired_separator_keywords_are_refused() {
    for retired in ["hyphen", "comma", "-", ","] {
        let out = encode(&["--separator", retired]);
        assert_eq!(
            out.status.code(),
            Some(64),
            "--separator {retired} must be refused; stderr={}",
            stderr_of(&out)
        );
        let stderr = stderr_of(&out);
        assert!(
            stderr.contains("Use `--separator space`"),
            "the refusal must name what replaced it; stderr={stderr:?}"
        );
    }
}

/// The control: whitespace grouping still works, by keyword and by literal.
#[test]
fn whitespace_separator_still_accepted_both_spellings() {
    for spelling in ["space", " "] {
        let out = encode(&["--group-size", "5", "--separator", spelling]);
        assert!(
            out.status.success(),
            "--separator {spelling:?} must still work; stderr={}",
            stderr_of(&out)
        );
        let artifact = first_line(&encode(&[]));
        assert_eq!(
            first_line(&out),
            regroup(&artifact, 5, ' '),
            "space-grouped stdout"
        );
    }
}
