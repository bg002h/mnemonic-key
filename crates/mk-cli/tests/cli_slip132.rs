//! Integration tests for `mk encode` SLIP-0132 prefix acceptance (A2).
//!
//! Verifies that zpub/ypub/etc. inputs are normalized to canonical xpub/tpub,
//! that the decoded card's xpub matches the canonical form, and that the
//! expected stderr note is emitted.

use std::process::Command;
use assert_cmd::cargo::CommandCargoExt;
use bitcoin::base58;

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

/// Invoke `mk encode` and decode the resulting mk1 strings via mk_codec.
fn run_encode_decode(xpub_arg: &str) -> (std::process::Output, mk_codec::KeyCard) {
    let out = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", xpub_arg, "--origin-path", "m/84h/0h/0h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output().unwrap();
    assert!(out.status.success(), "mk encode failed: stderr={}", String::from_utf8_lossy(&out.stderr));
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
    assert!(stderr.contains(NOTE_ZPUB), "missing SLIP-0132 note; stderr={stderr}");

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
    let out = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", &zpub, "--origin-path", "m/49h/0h/0h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(code, 64, "expected exit 64 (UsageError), got {code}; stderr={stderr}");
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
    let out = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", &zpub_multisig, "--origin-path", "m/48h/0h/0h/2h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(code, 0, "expected exit 0 for matching Zpub multisig; stderr={stderr}");
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
    let out = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", &zpub_multisig, "--origin-path", "m/48h/0h/0h/1h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(code, 64, "expected exit 64 for index mismatch; stderr={stderr}");
    assert!(
        stderr.contains("SLIP-0132/origin-path mismatch"),
        "expected mismatch message; stderr={stderr}"
    );
}

/// A canonical xpub (no SLIP-0132 prefix) must exit 0 and must NOT emit any
/// SLIP-0132 note on stderr.
#[test]
fn encode_canonical_xpub_no_note() {
    let out = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", V2_84_MAIN, "--origin-path", "m/84h/0h/0h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output()
        .unwrap();
    let code = out.status.code().unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(code, 0, "expected exit 0 for canonical xpub; stderr={stderr}");
    assert!(
        !stderr.contains("SLIP-0132"),
        "canonical xpub must not emit a SLIP-0132 note; stderr={stderr}"
    );
}
