//! Baseline GAP-B mutation-survivor kills (tests only): `mk verify`
//! `--origin-fingerprint` / `--origin-path` accept and refuse paths, the
//! xpub-fingerprint and policy-stub text printed by `decode` / `inspect` /
//! `verify --json`.
//!
//! The card is the committed V1 xpub (`from_md1_set.rs`) with origin
//! fingerprint 73c5da0a, path m/48'/0'/0'/2' and policy-id stub 3d190af3,
//! encoded by `mk encode` (two strings, chunk-set-id 647f1).

use std::process::{Command, Output};

use assert_cmd::cargo::CommandCargoExt;

const V1_XPUB: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const S1: &str = "mk1qpv3l3pqqsqn6xg27deutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5w9geez9gy5cyrj7";
const S2: &str = "mk1qpv3l3pp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl99j87ypa0zjpylkyqqrax";
const STUB: &str = "3d190af3";
/// Fingerprint of the xpub itself (not the origin master fingerprint).
const XPUB_FP: &str = "abc63537";

fn mk(args: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .expect("mk binary")
        .args(args)
        .output()
        .unwrap()
}

fn text(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

/// The same card minted with `--privacy-preserving` (no origin fingerprint).
fn privacy_card() -> Vec<String> {
    let out = mk(&[
        "encode",
        "--xpub",
        V1_XPUB,
        "--privacy-preserving",
        "--origin-path",
        "m/48'/0'/0'/2'",
        "--policy-id-stub",
        STUB,
    ]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    text(&out)
        .lines()
        .filter(|l| l.starts_with("mk1"))
        .map(str::to_string)
        .collect()
}

#[test]
fn verify_origin_fingerprint_right_wrong_omitted() {
    // Right fingerprint: exit 0.
    let ok = mk(&["verify", "--origin-fingerprint", "73c5da0a", S1, S2]);
    assert_eq!(ok.status.code(), Some(0), "{:?}", ok);
    // Wrong fingerprint: exit 1 (NO) naming the field.
    let bad = mk(&["verify", "--origin-fingerprint", "73c5da0b", S1, S2]);
    assert_eq!(bad.status.code(), Some(1), "{:?}", bad);
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("origin_fingerprint"),
        "{:?}",
        bad
    );
    // Card minted without a fingerprint: any expected fingerprint is exit 1.
    let cards = privacy_card();
    let mut args = vec!["verify", "--origin-fingerprint", "73c5da0a"];
    args.extend(cards.iter().map(String::as_str));
    let none = mk(&args);
    assert_eq!(none.status.code(), Some(1), "{:?}", none);
    assert!(
        String::from_utf8_lossy(&none.stderr).contains("omitted"),
        "{:?}",
        none
    );
}

#[test]
fn verify_origin_path_right_wrong() {
    let ok = mk(&["verify", "--origin-path", "m/48'/0'/0'/2'", S1, S2]);
    assert_eq!(ok.status.code(), Some(0), "{:?}", ok);
    let bad = mk(&["verify", "--origin-path", "m/84'/0'/0'", S1, S2]);
    assert_eq!(bad.status.code(), Some(1), "{:?}", bad);
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("origin_path"),
        "{:?}",
        bad
    );
}

#[test]
fn inspect_prints_xpub_fingerprint_and_stub() {
    let out = mk(&["inspect", S1, S2]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let t = text(&out);
    assert!(
        t.contains(&format!("xpub_fingerprint:    {XPUB_FP}\n")),
        "{t:?}"
    );
    assert!(
        t.contains(&format!("policy_id_stubs:     {STUB}\n")),
        "{t:?}"
    );

    let j = mk(&["inspect", "--json", S1, S2]);
    assert_eq!(j.status.code(), Some(0), "{:?}", j);
    let v: serde_json::Value = serde_json::from_slice(&j.stdout).unwrap();
    assert_eq!(v["xpub_fingerprint"], XPUB_FP);
    assert_eq!(v["policy_id_stubs"][0], STUB);
}

#[test]
fn decode_prints_policy_stub() {
    let out = mk(&["decode", S1, S2]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    assert!(
        text(&out).contains(&format!("policy_id_stubs:     {STUB}\n")),
        "{:?}",
        out
    );
    let j = mk(&["decode", "--json", S1, S2]);
    let v: serde_json::Value = serde_json::from_slice(&j.stdout).unwrap();
    assert_eq!(v["policy_id_stubs"][0], STUB);
}

#[test]
fn verify_json_prints_policy_stub() {
    let out = mk(&["verify", "--json", S1, S2]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["policy_id_stubs"][0], STUB);
}
