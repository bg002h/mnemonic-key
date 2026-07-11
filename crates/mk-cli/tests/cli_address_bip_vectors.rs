//! T4-a — spec-EXTERNAL oracle for `mk address` rendering
//! (constellation-eval §2 item #9; `design/SPEC_test_hardening_T4_mk_external_oracle.md`).
//!
//! `cli_address.rs`'s expected addresses were computed by the toolkit's
//! `address_search.rs`, which mk's `derive_support.rs` was copy-derived from
//! (`derive_support.rs:1-8`, shared lineage). A semantic mistake copied into
//! BOTH (wrong purpose→script-type arm, wrong HRP) would be invisible to that
//! cross-check — the frozen literals could be wrong-at-birth. These vectors
//! are instead the PUBLISHED BIP-84/BIP-86 first-address test vectors, fixed
//! by the standard before mk or the toolkit existed, so they cannot share
//! that provenance defect.
//!
//! Ingested via the REAL CLI path: `mk encode --xpub <published account key>`
//! (SLIP-0132 zpub→xpub normalization is wired at `cmd/mod.rs::parse_xpub_normalized`,
//! already exercised by `cli_slip132.rs`), then `mk address` on the emitted
//! card. mk-cli is bin-only (no lib target) — driven via `assert_cmd`, the
//! `cli_address.rs:56-64` idiom, never a library render path.

use std::process::Output;

use assert_cmd::Command;

// --- BIP-84 (P2WPKH, m/84'/0'/0') published vector ------------------------
// Source: https://raw.githubusercontent.com/bitcoin/bips/master/bip-0084.mediawiki
// "Test vectors" section (mnemonic `abandon abandon ... about`). Re-verified
// character-for-character against the live BIP text 2026-07-10.
const BIP84_ACCOUNT_ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
/// m/84'/0'/0'/0/0
const BIP84_RECEIVE_0: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
/// m/84'/0'/0'/0/1
const BIP84_RECEIVE_1: &str = "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g";
/// m/84'/0'/0'/1/0
const BIP84_CHANGE_0: &str = "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el";

// --- BIP-86 (P2TR, m/86'/0'/0') published vector --------------------------
// Source: https://raw.githubusercontent.com/bitcoin/bips/master/bip-0086.mediawiki
// "Test vectors" section (same mnemonic). Re-verified character-for-character
// against the live BIP text 2026-07-10.
const BIP86_ACCOUNT_XPUB: &str = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
/// m/86'/0'/0'/0/0
const BIP86_RECEIVE_0: &str = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";
/// m/86'/0'/0'/0/1
const BIP86_RECEIVE_1: &str = "bc1p4qhjn9zdvkux4e44uhx8tc55attvtyu358kutcqkudyccelu0was9fqzwh";
/// m/86'/0'/0'/1/0
const BIP86_CHANGE_0: &str = "bc1p3qkhfews2uk44qtvauqyr2ttdsw7svhkl9nkm9s9c3x4ax5h60wqwruhk7";

/// Run `mk encode --xpub <key> --origin-path <path>` and return ALL emitted
/// mk1 chunk lines. **CRITICAL (R0 M-r2-1):** an account-depth card can chunk
/// into more than one mk1 string — callers MUST forward every line to
/// `mk address`, never just the first (`lines().next()`).
fn encode_card(xpub: &str, origin_path: &str) -> Vec<String> {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            xpub,
            "--origin-path",
            origin_path,
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
            "--group-size",
            "0",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "mk encode failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lines: Vec<String> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert!(!lines.is_empty(), "mk encode emitted no mk1 strings");
    lines
}

fn run_address(cardstrs: &[String], extra: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .unwrap()
        .arg("address")
        .args(cardstrs)
        .args(extra)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}
fn stderr(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}

/// Extract rendered-address tokens (the trailing whitespace-delimited field
/// of an index line, e.g. "  0  bc1q..."), skipping chain-header lines like
/// "receive (m/0/i):" which don't end in a bech32 token.
fn rendered_addresses<'a>(out: &'a str, hrp_prefix: &str) -> Vec<&'a str> {
    out.lines()
        .filter_map(|l| l.split_whitespace().last())
        .filter(|tok| tok.starts_with(hrp_prefix))
        .collect()
}

#[test]
fn bip84_published_vector_matches_verbatim() {
    let card = encode_card(BIP84_ACCOUNT_ZPUB, "m/84'/0'/0'");
    // Provenance note (not a hard gate on the exact count): the account card
    // is expected to chunk into multiple mk1 strings; forwarding ALL of them
    // (not just the first) is the point of this fixture — see doc comment.
    assert!(
        !card.is_empty(),
        "encode_card must capture at least one mk1 chunk"
    );

    let o = run_address(&card, &["--count", "2", "--chain", "both"]);
    assert_eq!(o.status.code(), Some(0), "stderr={}", stderr(&o));
    let out = stdout(&o);
    let addrs = rendered_addresses(&out, "bc1q");
    assert!(
        addrs.contains(&BIP84_RECEIVE_0),
        "missing published BIP-84 receive/0 {BIP84_RECEIVE_0}; rendered={addrs:?}"
    );
    assert!(
        addrs.contains(&BIP84_RECEIVE_1),
        "missing published BIP-84 receive/1 {BIP84_RECEIVE_1}; rendered={addrs:?}"
    );
    assert!(
        addrs.contains(&BIP84_CHANGE_0),
        "missing published BIP-84 change/0 {BIP84_CHANGE_0}; rendered={addrs:?}"
    );
}

#[test]
fn bip86_published_vector_matches_verbatim() {
    let card = encode_card(BIP86_ACCOUNT_XPUB, "m/86'/0'/0'");
    assert!(
        !card.is_empty(),
        "encode_card must capture at least one mk1 chunk"
    );

    let o = run_address(&card, &["--count", "2", "--chain", "both"]);
    assert_eq!(o.status.code(), Some(0), "stderr={}", stderr(&o));
    let out = stdout(&o);
    let addrs = rendered_addresses(&out, "bc1p");
    assert!(
        addrs.contains(&BIP86_RECEIVE_0),
        "missing published BIP-86 receive/0 {BIP86_RECEIVE_0}; rendered={addrs:?}"
    );
    assert!(
        addrs.contains(&BIP86_RECEIVE_1),
        "missing published BIP-86 receive/1 {BIP86_RECEIVE_1}; rendered={addrs:?}"
    );
    assert!(
        addrs.contains(&BIP86_CHANGE_0),
        "missing published BIP-86 change/0 {BIP86_CHANGE_0}; rendered={addrs:?}"
    );
}
