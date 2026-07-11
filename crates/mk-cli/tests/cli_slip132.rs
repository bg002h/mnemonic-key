//! Integration tests for `mk encode`/`mk verify` SLIP-0132 prefix acceptance (A2/A3).
//!
//! Verifies that zpub/ypub/etc. inputs are normalized to canonical xpub/tpub,
//! that the decoded card's xpub matches the canonical form, that the
//! expected stderr note is emitted, and that `mk verify` accepts SLIP-0132
//! prefixes (path-optional) with correct exit codes.

use assert_cmd::cargo::CommandCargoExt;
use bitcoin::base58;
use std::process::Command;

const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
/// Depth-4 m/48'/0'/0'/2' account xpub (BIP-48 P2WSH multisig).
const V1_48_MULTISIG: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";

/// Re-version a canonical xpub into a SLIP-0132 form (inverse of normalize).
fn to_slip132(xpub_str: &str, version: [u8; 4]) -> String {
    let mut data = base58::decode_check(xpub_str).unwrap();
    data[0..4].copy_from_slice(&version);
    base58::encode_check(&data)
}
const ZPUB_V: [u8; 4] = [0x04, 0xB2, 0x47, 0x46];
const NOTE_ZPUB: &str = "note: --xpub was a SLIP-0132 zpub";
/// Zpub = BIP-48 P2WSH multisig mainnet.
const ZPUB_MULTISIG_V: [u8; 4] = [0x02, 0xAA, 0x7E, 0xD3];
const WATCH_ONLY: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend";

/// Build a real mk1 card (from V2_84_MAIN with a known origin path) and return
/// the chunk strings ready to pass as positional args to `mk verify`.
fn make_card() -> Vec<String> {
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
        "make_card: mk encode failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

/// Invoke `mk encode` and decode the resulting mk1 strings via mk_codec.
fn run_encode_decode(xpub_arg: &str) -> (std::process::Output, mk_codec::KeyCard) {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            xpub_arg,
            "--origin-path",
            "m/84h/0h/0h",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
            // mstring-grouping P3: these stdout lines go straight to mk_codec::decode
            // (bypassing the CLI intake strip) — keep them unbroken.
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
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    let strings: Vec<String> = stdout.lines().map(str::to_string).collect();
    assert!(!strings.is_empty(), "no mk1 strings on stdout");
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs).expect("decode emitted mk1 strings");
    (out, card)
}

#[test]
fn encode_accepts_zpub_with_matching_path() {
    let zpub = to_slip132(V2_84_MAIN, ZPUB_V);

    let (zpub_out, zpub_card) = run_encode_decode(&zpub);

    // stderr must contain the SLIP-0132 normalization note
    let stderr = String::from_utf8(zpub_out.stderr).unwrap();
    assert!(
        stderr.contains(NOTE_ZPUB),
        "missing SLIP-0132 note; stderr={stderr}"
    );

    // The decoded xpub must equal the canonical xpub.
    // (mk encode is non-deterministic due to random chunk_set_id; we compare
    // decoded card fields, not raw mk1 byte strings.)
    let (_, canon_card) = run_encode_decode(V2_84_MAIN);
    assert_eq!(
        zpub_card.xpub, canon_card.xpub,
        "zpub-derived card xpub must equal canonical xpub-derived card xpub"
    );
}

/// A zpub passed with a mismatching --origin-path (purpose 49' instead of 84') must
/// exit 64 with a SLIP-0132/origin-path mismatch message naming the expected purpose.
#[test]
fn encode_zpub_path_mismatch_refuses() {
    let zpub = to_slip132(V2_84_MAIN, ZPUB_V);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            &zpub,
            "--origin-path",
            "m/49h/0h/0h",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
        ])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(
        code, 64,
        "expected exit 64 (UsageError), got {code}; stderr={stderr}"
    );
    assert!(
        stderr.contains("SLIP-0132/origin-path mismatch"),
        "expected mismatch message in stderr; stderr={stderr}"
    );
    assert!(
        stderr.contains("expects --origin-path purpose 84'"),
        "expected 'expects --origin-path purpose 84'' in stderr; stderr={stderr}"
    );
}

/// A Zpub (BIP-48 P2WSH multisig) with the matching m/48'/0'/0'/2' path must
/// exit 0 and emit the SLIP-0132 Zpub normalization note on stderr.
#[test]
fn encode_zpub_multisig_match() {
    let zpub_multisig = to_slip132(V1_48_MULTISIG, ZPUB_MULTISIG_V);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            &zpub_multisig,
            "--origin-path",
            "m/48h/0h/0h/2h",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
        ])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(
        code, 0,
        "expected exit 0 for matching Zpub multisig; stderr={stderr}"
    );
    assert!(
        stderr.contains("note: --xpub was a SLIP-0132 Zpub"),
        "expected Zpub normalization note; stderr={stderr}"
    );
}

/// A Zpub (BIP-48 P2WSH multisig) with a mismatching script-type index (1' not 2')
/// must exit 64 with a SLIP-0132/origin-path mismatch message.
#[test]
fn encode_zpub_multisig_index_mismatch() {
    let zpub_multisig = to_slip132(V1_48_MULTISIG, ZPUB_MULTISIG_V);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            &zpub_multisig,
            "--origin-path",
            "m/48h/0h/0h/1h",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
        ])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(
        code, 64,
        "expected exit 64 for index mismatch; stderr={stderr}"
    );
    assert!(
        stderr.contains("SLIP-0132/origin-path mismatch"),
        "expected mismatch message; stderr={stderr}"
    );
}

/// A canonical xpub (no SLIP-0132 prefix) must exit 0 and must NOT emit any
/// SLIP-0132 note on stderr.
#[test]
fn encode_canonical_xpub_no_note() {
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
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(
        code, 0,
        "expected exit 0 for canonical xpub; stderr={stderr}"
    );
    assert!(
        !stderr.contains("SLIP-0132"),
        "canonical xpub must not emit a SLIP-0132 note; stderr={stderr}"
    );
}

// ── A3: mk verify accepts SLIP-0132 prefixes ──────────────────────────────────

/// `mk verify --xpub <zpub>` (NO --origin-path) must exit 0 and emit the
/// SLIP-0132 note. The mismatch message must NOT appear.
#[test]
fn verify_zpub_without_path_ok() {
    let card = make_card();
    let zpub = to_slip132(V2_84_MAIN, ZPUB_V);
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.arg("verify");
    for chunk in &card {
        cmd.arg(chunk);
    }
    cmd.args(["--xpub", &zpub]);
    let out = cmd.output().unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(
        code, 0,
        "expected exit 0 (zpub without path); stderr={stderr}"
    );
    assert!(
        stderr.contains(NOTE_ZPUB),
        "expected SLIP-0132 note in stderr; stderr={stderr}"
    );
    assert!(
        !stderr.contains("SLIP-0132/origin-path mismatch"),
        "must NOT contain mismatch message; stderr={stderr}"
    );
}

/// `mk verify --xpub <zpub> --origin-path m/49h/0h/0h` must exit 64
/// (UsageError) and emit the SLIP-0132/origin-path mismatch message.
#[test]
fn verify_zpub_path_mismatch_refuses() {
    let card = make_card();
    let zpub = to_slip132(V2_84_MAIN, ZPUB_V);
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.arg("verify");
    for chunk in &card {
        cmd.arg(chunk);
    }
    cmd.args(["--xpub", &zpub, "--origin-path", "m/49h/0h/0h"]);
    let out = cmd.output().unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(
        code, 64,
        "expected exit 64 (UsageError), got {code}; stderr={stderr}"
    );
    assert!(
        stderr.contains("SLIP-0132/origin-path mismatch"),
        "expected mismatch message in stderr; stderr={stderr}"
    );
}

/// `mk encode --xpub <zpub>` must emit BOTH the SLIP-0132 note AND the
/// watch-only advisory, with the SLIP-0132 note appearing before the watch-only
/// line (SLIP-0132 fires at parse-time; watch-only fires after stdout emit).
#[test]
fn encode_emits_both_slip132_note_and_watchonly_advisory() {
    let zpub = to_slip132(V2_84_MAIN, ZPUB_V);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            &zpub,
            "--origin-path",
            "m/84h/0h/0h",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
        ])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(code, 0, "expected exit 0; stderr={stderr}");
    assert!(
        stderr.contains(NOTE_ZPUB),
        "missing SLIP-0132 note; stderr={stderr}"
    );
    assert!(
        stderr.contains(WATCH_ONLY),
        "missing watch-only advisory; stderr={stderr}"
    );
    let slip_offset = stderr.find(NOTE_ZPUB).unwrap();
    let watch_offset = stderr.find(WATCH_ONLY).unwrap();
    assert!(
        slip_offset < watch_offset,
        "SLIP-0132 note (offset {slip_offset}) must precede watch-only advisory (offset {watch_offset})"
    );
}

// ── T4-c: closes FOLLOWUP `mk-slip0132-byte-parity-test-self-referential` ────
//
// `slip132_version_bytes_match_slip0132` (in-crate `src/slip132.rs` unit test)
// self-referentially builds its own SLIP-0132 byte string by re-versioning a
// corpus xpub with a LOCAL constant, then checks `detect_and_normalize`
// against that SAME local constant — a coordinated edit to both the test's
// byte array and the production match arm would stay green (FOLLOWUP option
// (b) concern). Option (a)'s remedy: anchor on a PUBLISHED extended-key
// string whose version bytes are fixed by the real SLIP-0132/BIP-84 spec,
// not by any constant in this repo, so a coordinated edit cannot fake it.
//
// mk-cli is bin-only (no lib target) — `detect_and_normalize` cannot be
// called directly from an external integration test. This test instead
// observes equivalent behavior through the CLI: normalizing the PUBLISHED
// BIP-84 zpub via `mk encode --xpub` must agree, field-for-field, with
// independently re-versioning that SAME published string's own base58check
// payload to canonical xpub bytes (`to_slip132`, run in the opposite
// direction) — a check tied to the published string's own bytes, not to a
// separately-sourced corpus xpub.

/// Published BIP-84 account zpub (m/84'/0'/0'), mnemonic `abandon ... about`.
/// Source: https://raw.githubusercontent.com/bitcoin/bips/master/bip-0084.mediawiki
/// "Test vectors". Re-verified character-for-character against the live BIP
/// text 2026-07-10.
const BIP84_PUBLISHED_ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
/// BIP-32 mainnet xpub version prefix (`0x0488B21E`, "xpub").
const XPUB_MAINNET_V: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];

#[test]
fn published_bip84_zpub_normalizes_and_matches_own_version_swap() {
    // Independently byte-swap the PUBLISHED zpub's own payload to canonical
    // xpub version bytes — not a re-versioned corpus xpub, so this cannot
    // "cancel" against a coordinated edit to detect_and_normalize's table
    // the way the in-crate self-referential test's local-constant pair can.
    let canonical_via_manual_swap = to_slip132(BIP84_PUBLISHED_ZPUB, XPUB_MAINNET_V);

    let (encoded_out, from_zpub_card) = run_encode_decode(BIP84_PUBLISHED_ZPUB);
    let stderr = String::from_utf8(encoded_out.stderr).unwrap();
    assert!(
        stderr.contains(NOTE_ZPUB),
        "missing SLIP-0132 note for the published zpub; stderr={stderr}"
    );

    let (_, from_manual_card) = run_encode_decode(&canonical_via_manual_swap);
    assert_eq!(
        from_zpub_card.xpub, from_manual_card.xpub,
        "mk-cli's SLIP-0132 zpub normalization of the PUBLISHED BIP-84 zpub must \
         agree with an independently byte-swapped canonical form of that SAME \
         published key"
    );
}
