//! `mk encode` chunk_set_id: deterministic by default, pinnable by flag.
//!
//! The default is derived from the canonical bytecode (mk-codec
//! `derive_chunk_set_id`), which is what makes `mk encode` output comparable
//! byte-for-byte against another implementation of the format — and what SPEC
//! §2.5's "reuse the same value for all subsequent re-encodings of the same
//! card" requires of a stateless encoder.
//!
//! Before this landed, three invocations on identical inputs produced three
//! different cards on the wire.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// A card that CHUNKS — the flag and the derivation only exist on the chunked
/// path, so a single-string card would prove nothing.
const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";

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
        "--group-size",
        "0",
    ]);
    cmd.args(extra);
    cmd.output().unwrap()
}

fn stdout_lines(out: &std::process::Output) -> Vec<String> {
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone())
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn repeated_invocations_emit_identical_strings() {
    let a = stdout_lines(&encode(&[]));
    let b = stdout_lines(&encode(&[]));
    let c = stdout_lines(&encode(&[]));
    assert!(
        a.len() > 1,
        "this fixture must chunk or the test proves nothing; got {} line(s)",
        a.len()
    );
    assert_eq!(a, b, "run 1 and run 2 disagree");
    assert_eq!(b, c, "run 2 and run 3 disagree");
}

#[test]
fn an_explicit_chunk_set_id_overrides_the_derived_default() {
    let derived = stdout_lines(&encode(&[]));
    let pinned = stdout_lines(&encode(&["--chunk-set-id", "0x12345"]));
    assert_ne!(
        derived, pinned,
        "the pin did not change the output, so the flag is not wired up"
    );
    // Pinning is itself deterministic.
    let pinned_again = stdout_lines(&encode(&["--chunk-set-id", "0x12345"]));
    assert_eq!(pinned, pinned_again);
    // And bare hex works as well as 0x-prefixed.
    let bare = stdout_lines(&encode(&["--chunk-set-id", "12345"]));
    assert_eq!(pinned, bare, "0x12345 and 12345 must mean the same value");
}

#[test]
fn a_chunk_set_id_over_20_bits_is_refused() {
    let out = encode(&["--chunk-set-id", "0x100000"]);
    assert!(
        !out.status.success(),
        "0x100000 does not fit the 20-bit field and must be refused, not truncated"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("chunk-set-id") || stderr.contains("chunk_set_id"),
        "the refusal must name the flag; stderr={stderr}"
    );
}

#[test]
fn a_non_hex_chunk_set_id_is_refused() {
    let out = encode(&["--chunk-set-id", "nonsense"]);
    assert!(!out.status.success(), "non-hex must be refused");
}
