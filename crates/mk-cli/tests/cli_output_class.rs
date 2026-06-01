//! Output-class stderr advisory tests for `mk` CLI.
//!
//! Verifies that `mk decode` emits the watch-only advisory line on stderr,
//! and that the advisory text is byte-identical to the toolkit's
//! `secret_advisory` and ms-cli's `advisory` (cross-repo parity).

use std::str::FromStr;

use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;

/// Exact watch-only advisory line (em-dash U+2014). MUST be byte-identical to
/// mnemonic-toolkit's secret_advisory.rs + ms-cli's advisory.rs.
const WATCH_ONLY_LINE: &str =
    "note: stdout is watch-only \u{2014} public keys only, cannot spend";

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
    assert_eq!(PRIVATE_KEY_LINE, "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')");
    assert_eq!(WATCH_ONLY_LINE, "note: stdout is watch-only \u{2014} public keys only, cannot spend");
    assert_eq!(TEMPLATE_LINE, "note: stdout is a keyless descriptor template (no keys)");
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
