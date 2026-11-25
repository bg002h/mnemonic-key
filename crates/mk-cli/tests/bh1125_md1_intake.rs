//! Bug hunt 2026-11-25, mk#3 and mk#1 — md1 intake through the md-codec this
//! crate resolves from crates.io (0.42.0, the newest published).
//!
//! mk#3: that md-codec truncates a non-ASCII character to a byte, so `ű`
//! (U+0171) read as `q` and an md1 the sibling `md` refuses was accepted on
//! `--from-md1`, `--from-md1-set` and `mk verify --from-md1`. mk-cli now
//! refuses a non-ASCII character in an md1 string itself (exit 1, the error
//! md-codec 0.48.5 gives), until the pin can move.
//!
//! mk#1: that md-codec reads md1 wire version 4 only. A wire-version-8 card
//! (minted by a newer `md`) is still refused, but the error now names the
//! limit instead of a bare "expected 4".

use std::process::{Command, Output};

use assert_cmd::cargo::CommandCargoExt;

const X: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
/// Keyless `wpkh` template, stub 559e64b2.
const TPL: &str = "md1yqpqqxzq2qwfv8urt848e";
/// `TPL` with `q` replaced by U+0171 and `8` by U+0238 (truncate to q / 8).
const TPL_U0171: &str = "md1y\u{0171}pqqxzq2qwfv8urt848e";
const TPL_U0238: &str = "md1yqpqqxzq2qwfv8urt84\u{0238}e";
/// descriptor-mnemonic vector `liana_taproot.phrase.txt`: wire version 8.
const LIANA_V8: &str = "md1gzfdsssj5qqcreqygvtam2lm5fenw4v";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn desc(o: &Output) -> String {
    format!(
        "code={:?} stdout={:?} stderr={:?}",
        o.status.code(),
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn encode_from_md1(md1: &str) -> Output {
    mk().args([
        "encode",
        "--xpub",
        X,
        "--origin-path",
        "m/84'/0'/0'",
        "--origin-fingerprint",
        "73c5da0a",
        "--from-md1",
        md1,
    ])
    .output()
    .unwrap()
}

#[test]
fn ascii_template_still_binds() {
    let o = encode_from_md1(TPL);
    assert_eq!(o.status.code(), Some(0), "{}", desc(&o));
}

#[test]
fn non_ascii_md1_is_refused_on_from_md1() {
    for md1 in [TPL_U0171, TPL_U0238] {
        let o = encode_from_md1(md1);
        assert_eq!(o.status.code(), Some(1), "{md1}: {}", desc(&o));
        assert!(o.stdout.is_empty(), "{}", desc(&o));
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("not in codex32 alphabet"),
            "{}",
            desc(&o)
        );
    }
}

#[test]
fn non_ascii_md1_is_refused_on_from_md1_set() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("set.md1");
    std::fs::write(&p, format!("{TPL_U0171}\n")).unwrap();
    let o = mk()
        .args([
            "encode",
            "--xpub",
            X,
            "--origin-path",
            "m/84'/0'/0'",
            "--origin-fingerprint",
            "73c5da0a",
            "--from-md1-set",
            p.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1), "{}", desc(&o));
    assert!(o.stdout.is_empty(), "{}", desc(&o));
}

#[test]
fn non_ascii_md1_is_refused_on_verify() {
    let card = encode_from_md1(TPL);
    assert_eq!(card.status.code(), Some(0), "{}", desc(&card));
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("card.txt");
    std::fs::write(&p, &card.stdout).unwrap();
    let ok = mk()
        .args(["verify", "--in", p.to_str().unwrap(), "--from-md1", TPL])
        .output()
        .unwrap();
    assert_eq!(ok.status.code(), Some(0), "{}", desc(&ok));
    let o = mk()
        .args([
            "verify",
            "--in",
            p.to_str().unwrap(),
            "--from-md1",
            TPL_U0171,
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1), "{}", desc(&o));
    assert!(o.stdout.is_empty(), "{}", desc(&o));
}

#[test]
fn wire_version_8_names_the_limit() {
    let o = encode_from_md1(LIANA_V8);
    assert_eq!(o.status.code(), Some(1), "{}", desc(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("md1 input rejected: wire-format version mismatch: got 8, expected 4"),
        "{}",
        desc(&o)
    );
    assert!(
        err.contains("reads md1 wire version 4 only"),
        "{}",
        desc(&o)
    );
    assert!(err.contains("--policy-id-stub"), "{}", desc(&o));
}
