//! Integration tests for `mk derive` (read-only child-xpub derivation).
//!
//! Fixtures built in-process via `mk_codec::encode`; expected child xpubs are
//! independently derived with the `bitcoin` crate and compared to the CLI output.

use std::process::Output;
use std::str::FromStr;

use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::Secp256k1;
use mk_codec::KeyCard;

const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
const V1_48_MULTISIG: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";

fn p(s: &str) -> DerivationPath {
    DerivationPath::from_str(s).unwrap()
}

fn card(xpub: &str, origin_path: &str) -> Vec<String> {
    let kc = KeyCard::new(
        vec![[0xde, 0xad, 0xbe, 0xef]],
        Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])),
        p(origin_path),
        Xpub::from_str(xpub).unwrap(),
    );
    mk_codec::encode(&kc).unwrap()
}

fn run(cardstrs: &[String], extra: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .unwrap()
        .arg("derive")
        .args(cardstrs)
        .args(extra)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}
fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

/// Independently derive the expected child xpub via the `bitcoin` crate.
fn expected_child(parent: &str, rel: &str) -> Xpub {
    let secp = Secp256k1::verification_only();
    Xpub::from_str(parent)
        .unwrap()
        .derive_pub(&secp, &p(rel))
        .unwrap()
}

#[test]
fn relative_path_derivation_matches_bitcoin() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--path", "m/0/5", "--json"]);
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let exp = expected_child(V2_84_MAIN, "m/0/5");
    assert_eq!(v["child_xpub"], exp.to_string());
    assert_eq!(v["depth"], 5); // account depth 3 + 2
    assert_eq!(
        v["child_fingerprint"],
        hex::encode(exp.fingerprint().to_bytes())
    );
    assert_eq!(v["relative_path"], "m/0/5");
    assert_eq!(v["network"], "mainnet");
}

#[test]
fn index_sugar_equals_path_m0() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let by_index = run(&c, &["--index", "5", "--json"]);
    let by_path = run(&c, &["--path", "m/0/5", "--json"]);
    let vi: serde_json::Value = serde_json::from_str(&stdout(&by_index)).unwrap();
    let vp: serde_json::Value = serde_json::from_str(&stdout(&by_path)).unwrap();
    assert_eq!(vi["child_xpub"], vp["child_xpub"]);
}

#[test]
fn hardened_path_rejected() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--path", "m/0'/0"]);
    assert_eq!(code(&o), 64, "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stderr).contains("hardened"));
}

#[test]
fn both_path_and_index_is_group_error() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--path", "m/0/0", "--index", "0"]);
    assert_eq!(code(&o), 64, "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn neither_path_nor_index_is_group_error() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &[]);
    assert_eq!(code(&o), 64, "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn multisig_card_is_allowed_for_derive() {
    // mk derive does NOT refuse multisig cards (per-cosigner child derivation is legit).
    let c = card(V1_48_MULTISIG, "m/48'/0'/0'/2'");
    let o = run(&c, &["--path", "m/0/0"]);
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn child_xpub_roundtrips_through_encode() {
    // Composability smoke: the derived child xpub re-encodes as a valid card.
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--path", "m/0/5", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let child = v["child_xpub"].as_str().unwrap();
    let enc = Command::cargo_bin("mk")
        .unwrap()
        .args([
            "encode",
            "--xpub",
            child,
            "--origin-path",
            "m/84'/0'/0'/0/5",
            "--policy-id-stub",
            "deadbeef",
            "--privacy-preserving",
        ])
        .output()
        .unwrap();
    assert!(
        enc.status.success(),
        "{}",
        String::from_utf8_lossy(&enc.stderr)
    );
    assert!(String::from_utf8_lossy(&enc.stdout).contains("mk1"));
}
