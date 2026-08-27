//! `mk encode` display-grouping flags (P3, SPEC §6a/§6b/§6c).
//!
//! **The grouped form moved off stdout.** §6a rules that `encode`'s stdout is
//! "the canonical artifact, ungrouped, nothing else"; §6b rules that
//! `--group-size` / `--separator` "affect the stderr card only"; §6c requires
//! `md` and `mk` — which had no card at all, only a one-line `note:` — to grow
//! one, and says the minimum it must carry is the grouped string itself, since
//! after this change that is the only place it exists.
//!
//! So the assertions below are a PAIR at every step: what left stdout, and
//! where it went. A test that only checked stdout would pass equally against an
//! implementation that deleted the grouped form outright.
//!
//! `--json` is UNCHANGED and out of scope this cycle (§6b).

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

/// `--group-size 5` and `--group-size 0` produce BYTE-IDENTICAL stdout.
///
/// This is the assertion that says the flag no longer reaches stdout at all,
/// rather than that one particular default happens to be unbroken.
#[test]
fn group_size_no_longer_reaches_stdout() {
    let grouped = encode(&["--group-size", "5"]);
    let unbroken = encode(&["--group-size", "0"]);
    assert!(grouped.status.success() && unbroken.status.success());
    assert_eq!(
        grouped.stdout, unbroken.stdout,
        "--group-size must not change stdout (SPEC §6b: the stderr card only)"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// §6c — the engraving card, on stderr.

// ──────────────────────────────────────────────────────────────────────────

/// The card carries the grouped form, then its two settings, then the advisory.
///
/// The grouped line is recomputed from the stdout artifact, so this fails if
/// the card renders a DIFFERENT string as well as if it renders none.
#[test]
fn the_stderr_card_carries_the_grouped_form_by_default() {
    let out = encode(&[]);
    assert!(out.status.success(), "stderr={}", stderr_of(&out));
    let stdout = stdout_of(&out);
    let stderr = stderr_of(&out);

    for artifact in stdout.lines() {
        let want = regroup(artifact, 5, ' ');
        assert!(
            stderr.lines().any(|l| l == want),
            "card must carry the space-5 grouped form of {artifact:?}; stderr={stderr:?}"
        );
    }
    assert!(
        stderr.lines().any(|l| l == "group size: 5"),
        "card names its group size; stderr={stderr:?}"
    );
    assert!(
        stderr.lines().any(|l| l == "separator: space"),
        "card names its separator; stderr={stderr:?}"
    );
    assert_eq!(
        stderr.lines().last(),
        Some(ADVISORY),
        "the existing advisory stays LAST; stderr={stderr:?}"
    );
}

/// `--group-size 0` yields a card whose artifact line is unbroken.
///
/// The card is still emitted: an operator who asked for no grouping still needs
/// to see what to transcribe, and `--no-engraving-card` is deliberately NOT
/// added to `mk` (§6c names it for `ms` and `mnemonic` only).
#[test]
fn group_size_0_makes_the_card_unbroken_but_still_a_card() {
    let out = encode(&["--group-size", "0"]);
    assert!(out.status.success(), "stderr={}", stderr_of(&out));
    let artifact = first_line(&out);
    let stderr = stderr_of(&out);
    assert!(
        stderr.lines().any(|l| l == artifact),
        "card carries the unbroken string verbatim; stderr={stderr:?}"
    );
    assert!(
        stderr.lines().any(|l| l == "group size: 0"),
        "card reports the group size it was given; stderr={stderr:?}"
    );
    assert_eq!(stderr.lines().last(), Some(ADVISORY));
}

/// `--json` is out of scope (§6b): the JSON envelope stays the ONLY thing on
/// stdout, and no card is printed in front of it.
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
            stderr.contains("space"),
            "the refusal must name what replaced it; stderr={stderr:?}"
        );
    }
}

/// The control: whitespace grouping still works, by keyword and by literal.
#[test]
fn whitespace_separator_still_accepted_both_spellings() {
    for spelling in ["space", " "] {
        let out = encode(&["--separator", spelling]);
        assert!(
            out.status.success(),
            "--separator {spelling:?} must still work; stderr={}",
            stderr_of(&out)
        );
        let artifact = first_line(&out);
        let stderr = stderr_of(&out);
        assert!(
            stderr.lines().any(|l| l == regroup(&artifact, 5, ' ')),
            "space-grouped card; stderr={stderr:?}"
        );
    }
}
