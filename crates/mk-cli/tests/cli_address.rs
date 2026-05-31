//! Integration tests for `mk address` (read-only address derivation).
//!
//! Fixtures are built in-process via `mk_codec::encode` (the `round_trip.rs`
//! idiom), and expected addresses are independently computed by the toolkit's
//! `mnemonic convert --to address` (cross-tool, not self-referential). Xpubs are
//! lifted from the mk-codec v0.1 corpus; the leaf fixture is forward-derived.

use std::process::Output;
use std::str::FromStr;

use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::Secp256k1;
use mk_codec::KeyCard;

// --- corpus xpubs (verified xpub↔origin_path pairs) -----------------------
const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
const V9_44_MAIN: &str = "xpub6BmeGmTooeHReVcjsVUoL7d1Jqo1qVgr8yQ9miszNgq1dYa7REWdbS3tKiSx1zpyBKvcXE2hDn7HJytBjgynVDiY1XbpX5JPNFLQv6SGuyA";
const V1_48_MULTISIG: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V8_87_MULTISIG: &str = "xpub6BmeGmTX8KDVHvTZuNUmLJ2t82Md7abNfb4DGD8ivFPnQo5gJn3TX5JMCC1GxeQNW7DpRrYaSW3goEADYFykhfic2iPHTjW6BquJqTdCbVu";
const V5_NONSTD: &str = "xpub6Den8YxgJdggPygKKEv3wiQwQ6PSGUouW98xC4obAJAqvuWcBMHuxeuXHxyZtAJHLqE7U1JdEXrNwbNPNCn1F79n4ZuBTLnzF7mPbLR3ZvB";
const V15_84_TEST: &str = "tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X";

// --- expected addresses (independently computed by toolkit `mnemonic convert`) ---
const V2_84_M0_0_P2WPKH: &str = "bc1qfjxgzvdwrxh9ejp6jmdlr9tc6lfl6adcsx2z4f";
const V2_84_M0_0_P2TR: &str = "bc1p5eyw3l25j96706xxfasv7qjca7mw6lyctwqn2mwrsxs9agpk70gswn99l2";
const V9_44_M0_0_P2PKH: &str = "1KyxX3KkXCFH6WZJhGNznNtN6dtcgP9ceY";

fn p(s: &str) -> DerivationPath {
    DerivationPath::from_str(s).unwrap()
}

/// Build a card (`mk1` chunk strings) for a known xpub + origin_path. The xpub's
/// depth/terminal-child must match `origin_path` (codec invariant), so callers
/// pass corpus pairs or forward-derived leaves.
fn card(xpub: &str, origin_path: &str) -> Vec<String> {
    let kc = KeyCard::new(
        vec![[0xde, 0xad, 0xbe, 0xef]],
        Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])),
        p(origin_path),
        Xpub::from_str(xpub).unwrap(),
    );
    mk_codec::encode(&kc).unwrap()
}

/// Forward-derive a depth-5 leaf xpub from the V2 account (for the I2 leaf case).
fn leaf_84_xpub() -> String {
    let secp = Secp256k1::verification_only();
    Xpub::from_str(V2_84_MAIN)
        .unwrap()
        .derive_pub(&secp, &p("m/0/5"))
        .unwrap()
        .to_string()
}

fn run(cardstrs: &[String], extra: &[&str]) -> Output {
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
fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

#[test]
fn account_84_default_p2wpkh_count10() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &[]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(stdout(&o).lines().filter(|l| l.contains("bc1q")).count(), 10);
}

#[test]
fn account_84_first_address_matches_toolkit() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--count", "1"]);
    assert!(stdout(&o).contains(V2_84_M0_0_P2WPKH), "{}", stdout(&o));
}

#[test]
fn account_84_p2tr_override_matches_toolkit() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--address-type", "p2tr", "--count", "1"]);
    assert!(stdout(&o).contains(V2_84_M0_0_P2TR), "{}", stdout(&o));
}

#[test]
fn account_44_default_p2pkh_matches_toolkit() {
    let c = card(V9_44_MAIN, "m/44'/0'/0'");
    let o = run(&c, &["--count", "1"]);
    assert!(stdout(&o).contains(V9_44_M0_0_P2PKH), "{}", stdout(&o));
}

#[test]
fn address_type_override_on_84_yields_legacy() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--address-type", "p2pkh", "--count", "1"]);
    // legacy P2PKH addresses start with '1'
    assert!(stdout(&o).lines().any(|l| l.trim_start().contains("  1")
        || l.split_whitespace().last().is_some_and(|a| a.starts_with('1'))));
}

#[test]
fn multisig_48_refused_even_with_address_type() {
    let c = card(V1_48_MULTISIG, "m/48'/0'/0'/2'");
    let o = run(&c, &["--address-type", "p2wpkh"]);
    assert_eq!(code(&o), 64, "{}", stderr(&o));
    assert!(stderr(&o).contains("multisig"), "{}", stderr(&o));
}

#[test]
fn multisig_87_refused() {
    let c = card(V8_87_MULTISIG, "m/87'/0'/0'");
    let o = run(&c, &[]);
    assert_eq!(code(&o), 64, "{}", stderr(&o));
}

#[test]
fn nonstandard_requires_address_type_then_succeeds() {
    let c = card(V5_NONSTD, "m/9999'/1234'/56'/7'");
    let no_flag = run(&c, &["--count", "1"]);
    assert_eq!(code(&no_flag), 64, "{}", stderr(&no_flag));
    assert!(stderr(&no_flag).contains("address-type"), "{}", stderr(&no_flag));
    let with_flag = run(&c, &["--address-type", "p2wpkh", "--count", "1"]);
    assert_eq!(code(&with_flag), 0, "{}", stderr(&with_flag));
}

#[test]
fn leaf_card_requires_then_advises_depth() {
    let leaf = leaf_84_xpub();
    let c = card(&leaf, "m/84'/0'/0'/0/5");
    // no --address-type → infer Unknown (len 5 ≠ 3) → exit 64
    let no_flag = run(&c, &["--count", "1"]);
    assert_eq!(code(&no_flag), 64, "{}", stderr(&no_flag));
    // with --address-type → succeeds, but emits the depth advisory on stderr
    let with_flag = run(&c, &["--address-type", "p2wpkh", "--count", "1"]);
    assert_eq!(code(&with_flag), 0, "{}", stderr(&with_flag));
    assert!(stderr(&with_flag).contains("depth"), "{}", stderr(&with_flag));
}

#[test]
fn range_inclusive_and_validation() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let ok = run(&c, &["--range", "2,4"]);
    assert_eq!(code(&ok), 0, "{}", stderr(&ok));
    assert_eq!(stdout(&ok).lines().filter(|l| l.contains("bc1q")).count(), 3); // 2,3,4
    let backwards = run(&c, &["--range", "5,2"]);
    assert_eq!(code(&backwards), 64, "{}", stderr(&backwards));
}

#[test]
fn count_range_conflict_exits_64() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--count", "1", "--range", "0,2"]);
    // clap parse conflict → mk-cli main.rs catch-all → exit 64
    assert_eq!(code(&o), 64, "{}", stderr(&o));
}

#[test]
fn chain_both_grouped_receive_then_change() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--chain", "both", "--count", "1"]);
    let out = stdout(&o);
    let recv = out.find("receive").unwrap();
    let chng = out.find("change").unwrap();
    assert!(recv < chng, "receive must precede change: {out}");
}

#[test]
fn network_inferred_testnet_override_regtest_and_mismatch() {
    let c = card(V15_84_TEST, "m/84'/1'/0'");
    // inferred testnet → tb1
    let inferred = run(&c, &["--count", "1"]);
    assert!(stdout(&inferred).contains("tb1q"), "{}", stdout(&inferred));
    // --network regtest → bcrt1 (test-kind, allowed)
    let regtest = run(&c, &["--network", "regtest", "--count", "1"]);
    assert!(stdout(&regtest).contains("bcrt1q"), "{}", stdout(&regtest));
    // --network mainnet on a test xpub → mismatch → exit 64
    let mismatch = run(&c, &["--network", "mainnet", "--count", "1"]);
    assert_eq!(code(&mismatch), 64, "{}", stderr(&mismatch));
}

#[test]
fn json_shape() {
    let c = card(V2_84_MAIN, "m/84'/0'/0'");
    let o = run(&c, &["--count", "2", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["address_type"], "p2wpkh");
    assert_eq!(v["network"], "mainnet");
    let addrs = v["addresses"].as_array().unwrap();
    assert_eq!(addrs.len(), 2);
    assert_eq!(addrs[0]["chain"], 0);
    assert_eq!(addrs[0]["index"], 0);
    assert_eq!(addrs[0]["address"], V2_84_M0_0_P2WPKH);
}
