//! Output-class stderr advisory tests for `mk` CLI.
//!
//! Verifies that `mk decode` / `mk encode` / `mk inspect` / `mk repair` /
//! `mk derive` / `mk address` all emit the watch-only advisory line on stderr,
//! and that the advisory text is byte-identical to the toolkit's
//! `secret_advisory` and ms-cli's `advisory` (cross-repo parity).

use std::str::FromStr;

use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;
use mk_codec::string_layer::bch::ALPHABET;

/// Exact watch-only advisory line (em-dash U+2014). MUST be byte-identical to
/// mnemonic-toolkit's secret_advisory.rs + ms-cli's advisory.rs.
const WATCH_ONLY_LINE: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend";

const PRIVATE_KEY_LINE: &str = "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')";
const TEMPLATE_LINE: &str = "note: stdout is a keyless descriptor template (no keys)";

/// Single-sig depth-3 account xpub (m/84'/0'/0'), lifted from cli_address.rs corpus.
const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

fn p(s: &str) -> DerivationPath {
    DerivationPath::from_str(s).unwrap()
}

/// Build a single-sig mk1 card — duplicated from cli_address.rs (integration-test crate boundary prevents sharing).
fn card(xpub: &str, origin_path: &str) -> Vec<String> {
    let kc = KeyCard::new(
        vec![[0xde, 0xad, 0xbe, 0xef]],
        Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])),
        p(origin_path),
        Xpub::from_str(xpub).unwrap(),
    );
    mk_codec::encode(&kc).unwrap()
}

/// The single-sig fixture all mk cells use (address-accepted, no depth advisory).
fn mk1_fixture() -> Vec<String> {
    card(V2_84_MAIN, "m/84h/0h/0h")
}

#[test]
fn byte_parity_advisory_lines() {
    assert_eq!(
        PRIVATE_KEY_LINE,
        "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')"
    );
    assert_eq!(
        WATCH_ONLY_LINE,
        "note: stdout is watch-only \u{2014} public keys only, cannot spend"
    );
    assert_eq!(
        TEMPLATE_LINE,
        "note: stdout is a keyless descriptor template (no keys)"
    );
}

#[test]
fn decode_emits_watch_only_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("decode")
        .args(mk1_fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "missing advisory; stderr={stderr}"
    );
}

#[test]
fn encode_emits_watch_only_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            V2_84_MAIN,
            "--origin-path",
            "m/84h/0h/0h",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "encode exited non-zero; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "missing advisory; stderr={stderr}"
    );
}

#[test]
fn inspect_emits_watch_only_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("inspect")
        .args(mk1_fixture())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "inspect exited non-zero; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "missing advisory; stderr={stderr}"
    );
}

#[test]
fn derive_emits_watch_only_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("derive")
        .args(mk1_fixture())
        .args(["--index", "0"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "derive exited non-zero; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "missing advisory; stderr={stderr}"
    );
}

#[test]
fn address_emits_watch_only_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("address")
        .args(mk1_fixture())
        .args(["--count", "1"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "address exited non-zero; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "missing advisory; stderr={stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Inert-subcommand negative cells
// ──────────────────────────────────────────────────────────────────────────────

/// Assert that none of the three advisory lines appear in `stderr`.
fn assert_no_advisory(stderr: &str) {
    for line in [PRIVATE_KEY_LINE, WATCH_ONLY_LINE, TEMPLATE_LINE] {
        assert!(
            !stderr.contains(line),
            "inert command emitted an advisory: {stderr}"
        );
    }
}

#[test]
fn verify_emits_no_advisory() {
    // `mk verify <mk1>` with no content matcher → BCH check only (inert).
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("verify")
        .args(mk1_fixture())
        .output()
        .unwrap();
    assert_no_advisory(&String::from_utf8(out.stderr).unwrap());
}

#[test]
fn vectors_emits_no_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("vectors")
        .output()
        .unwrap();
    assert_no_advisory(&String::from_utf8(out.stderr).unwrap());
}

#[test]
fn gui_schema_emits_no_advisory() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("gui-schema")
        .output()
        .unwrap();
    assert_no_advisory(&String::from_utf8(out.stderr).unwrap());
}

// ──────────────────────────────────────────────────────────────────────────────

/// Flip the bech32 character at position `pos` (0-indexed into the data
/// part, i.e. chars after the `mk1` separator). Returns a corrupted string
/// that is parseable but BCH-invalid. Copied from `cli_repair.rs::flip_at`.
fn flip_at(chunk: &str, pos: usize) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let alphabet_str = std::str::from_utf8(ALPHABET).unwrap();
    let was_idx = alphabet_str.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = alphabet_str.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

#[test]
fn repair_emits_watch_only_advisory() {
    // Corrupt one data-region symbol so BCH correction fires (exit 5).
    let chunks = mk1_fixture();
    let corrupted = flip_at(&chunks[0], 5);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", &corrupted])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(5),
        "expected exit 5 (correction applied); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "missing advisory; stderr={stderr}"
    );
}
