//! Integration tests for `mk-cli` v0.2.
//!
//! Realizes plan step 1.3.1.

use std::process::Command;
use std::str::FromStr;

use assert_cmd::cargo::CommandCargoExt;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use bitcoin::hashes::{Hash, sha256};
use mk_codec::KeyCard;

/// Canonical V1 fixture from `crates/mk-codec/src/test_vectors/v0.1.json`.
const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V1_FP_HEX: &str = "aabbccdd";
const V1_PATH: &str = "m/48'/0'/0'/2'";

/// Canonical md1 string from descriptor-mnemonic's pkh_basic.phrase.txt.
/// Used for the `--from-md1` derivation test.
/// Refreshed in mk-cli-v0.4.1 against md-codec v0.34.0 (v0.18+ wire-format).
const PKH_BASIC_MD1: &str = "md1yqpqqxzq2qwfv8urt848e";

#[test]
fn encode_decode_round_trip() {
    let xpub = Xpub::from_str(V1_XPUB).unwrap();
    let fp = Fingerprint::from([0xaa, 0xbb, 0xcc, 0xdd]);
    let path = DerivationPath::from_str(V1_PATH).unwrap();
    let stub = [0x11u8, 0x22, 0x33, 0x44];
    let card = KeyCard::new(vec![stub], Some(fp), path, xpub);

    let strings = mk_codec::encode(&card).expect("encode");
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let decoded = mk_codec::decode(&refs).expect("decode");

    assert_eq!(decoded.policy_id_stubs, card.policy_id_stubs);
    assert_eq!(decoded.origin_fingerprint, card.origin_fingerprint);
    assert_eq!(decoded.origin_path, card.origin_path);
    assert_eq!(decoded.xpub, card.xpub);
}

/// Confirm `mk encode --from-md1 <md1>` produces an mk1 string whose
/// decoded `policy_id_stubs[0]` equals `SHA-256(canonical_bytecode)[..4]`.
#[test]
fn from_md1_derivation() {
    // Reproduce the §3.5.1 derivation directly.
    let descriptor = md_codec::decode_md1_string(PKH_BASIC_MD1).expect("md1 decode");
    let (bytecode_bytes, _bit_len) = md_codec::encode_payload(&descriptor).expect("md1 payload");
    let hash = sha256::Hash::hash(&bytecode_bytes);
    let expected_stub: [u8; 4] = hash.as_byte_array()[..4].try_into().unwrap();

    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args([
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-fingerprint",
            V1_FP_HEX,
            "--origin-path",
            V1_PATH,
            "--from-md1",
            PKH_BASIC_MD1,
        ])
        .output()
        .expect("invoke mk encode");
    assert!(out.status.success(), "mk encode failed: {:?}", out);

    // Each line on stdout is one mk1 chunk.
    let stdout = String::from_utf8(out.stdout).unwrap();
    let strings: Vec<String> = stdout.lines().map(str::to_string).collect();
    assert!(!strings.is_empty(), "no mk1 strings on stdout");

    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs).expect("decode emitted strings");

    assert_eq!(card.policy_id_stubs.len(), 1);
    assert_eq!(card.policy_id_stubs[0], expected_stub);
}

/// `mk vectors --out <DIR>` must write one file per fixture without any
/// runtime path-dep on `crates/mk-codec/tests/`. Demonstrates `include_str!`
/// is producing self-contained binaries.
#[test]
fn vectors_subcommand_no_path_dep() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["vectors", "--out"])
        .arg(dir.path())
        .output()
        .expect("invoke mk vectors");
    assert!(out.status.success(), "mk vectors failed: {:?}", out);

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read tempdir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    assert!(
        entries.len() >= 8,
        "expected ≥8 fixture files, got {}",
        entries.len()
    );

    // Spot-check: every file is parseable JSON with a `name` field.
    for entry in entries {
        let body = std::fs::read_to_string(entry.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert!(parsed.get("name").is_some(), "missing name in {:?}", entry);
    }
}

/// `mk verify` with a wrong `--xpub` must exit 4 (ContentMismatch).
#[test]
fn verify_content_mismatch_exits_4() {
    // Build a real mk1 string from V1 first.
    let xpub = Xpub::from_str(V1_XPUB).unwrap();
    let fp = Fingerprint::from([0xaa, 0xbb, 0xcc, 0xdd]);
    let path = DerivationPath::from_str(V1_PATH).unwrap();
    let stub = [0x11u8, 0x22, 0x33, 0x44];
    let card = KeyCard::new(vec![stub], Some(fp), path, xpub);
    let strings = mk_codec::encode(&card).expect("encode");

    let wrong_xpub = "xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let mut args: Vec<String> = vec!["verify".into()];
    args.extend(strings.iter().cloned());
    args.push("--xpub".into());
    args.push(wrong_xpub.into());

    let out = cmd.args(&args).output().expect("invoke mk verify");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        4,
        "expected exit 4 ContentMismatch, got {code}; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
