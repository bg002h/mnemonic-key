//! FIX_mkc_keycard_validation rows C1-C9 (bughunt-1119 mk I8, I9; ruling T46):
//! mk encode refuses, mk verify reports invalid (exit 1), mk decode warns.
use assert_cmd::cargo::CommandCargoExt;
use std::process::{Command, Output};

const BAD_TPUB: &str = "tpubD6PRKLEwwo1MaZJYCnVjQHmEig2xBZVzkFQrrQFz5VA3GLdKmtYiB6tM7y1uTRthqM4dxRRK528DnR3qXiSopvZj238hq6e7Ek7WjNDscen";
const LEGACY_I8: [&str; 2] = [
    "mk1qpeffzpqqqqaatd7allqqpp4sl8szqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqnehen8u5yagl2mw0p43h3j",
    "mk1qpeffzppl8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqte3jjj9mtsugd0x98xyvntv",
];
const LEGACY_I9: [&str; 2] = [
    "mk1qp32rnpqqsqaatd7al02m0h0lcqqgdv8euqqqqqqqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsyr6hhd3h4f5yu7sp",
    "mk1qp32rnpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vg4peejgreyhlqwm83g90",
];
const SENTINEL_ZERO: [&str; 2] = [
    "mk1qp2enrpqqsqaatd7auqqqqqqlcqqgdv8euqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy8h8vagvc68yhng5",
    "mk1qp2enrpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v9ve3hu53hjkuceu46e8y",
];
const I8_ENCODE_ERR: &str = "error: xpub tpubD6PRKLEwwo1MaZJYCnVjQHmEig2xBZVzkFQrrQFz5VA3GLdKmtYiB6tM7y1uTRthqM4dxRRK528DnR3qXiSopvZj238hq6e7Ek7WjNDscen is at depth 0 but carries parent fingerprint 01010101; BIP 32 requires 00000000 for a master key, so the xpub is not a valid BIP 32 key and no card was minted.\n";
const I8_DISPLAY: &str =
    "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key";
const I9_DISPLAY: &str =
    "origin fingerprint deadbeef is not the depth-0 key's own fingerprint 751e76e8";

fn mk(args: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}
fn s(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

#[test]
fn kv_cli_i8_encode_refuses_invalid_master() {
    let o = mk(&[
        "encode",
        "--xpub",
        BAD_TPUB,
        "--origin-path",
        "m",
        "--privacy-preserving",
        "--policy-id-stub",
        "deadbeef",
    ]);
    assert_eq!(o.status.code(), Some(64), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(s(&o.stderr), I8_ENCODE_ERR);
}

#[test]
fn kv_cli_i8_encode_refuses_invalid_master_with_own_fp() {
    let o = mk(&[
        "encode",
        "--xpub",
        BAD_TPUB,
        "--origin-path",
        "m",
        "--origin-fingerprint",
        "751e76e8",
        "--policy-id-stub",
        "deadbeef",
    ]);
    assert_eq!(o.status.code(), Some(64), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stderr), I8_ENCODE_ERR);
}

#[test]
fn kv_cli_verify_reports_invalid_master_i8() {
    let o = mk(&["verify", LEGACY_I8[0], LEGACY_I8[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(s(&o.stderr), format!("error: {I8_DISPLAY}\n"));
}

#[test]
fn kv_cli_verify_json_reports_invalid_master_i9() {
    let o = mk(&["verify", "--json", LEGACY_I9[0], LEGACY_I9[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "MasterFingerprintMismatch");
    assert_eq!(v["error"]["message"], I9_DISPLAY);
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn kv_cli_decode_warns_on_invalid_master() {
    let o = mk(&["decode", LEGACY_I8[0], LEGACY_I8[1]]);
    assert_eq!(o.status.code(), Some(0));
    assert!(s(&o.stdout).starts_with(&format!("xpub:                {BAD_TPUB}\n")));
    let want = format!(
        "warning: this card is invalid ({I8_DISPLAY}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it\n"
    );
    assert!(s(&o.stderr).starts_with(&want), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_verify_sentinel_zero_fp_ok() {
    let o = mk(&["verify", SENTINEL_ZERO[0], SENTINEL_ZERO[1]]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_decode_sentinel_no_warning() {
    let o = mk(&["decode", SENTINEL_ZERO[0], SENTINEL_ZERO[1]]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        !s(&o.stderr).contains("this card is invalid"),
        "stderr: {}",
        s(&o.stderr)
    );
}

#[test]
fn kv_cli_decode_json_warns_on_invalid_master() {
    let o = mk(&["decode", "--json", LEGACY_I9[0], LEGACY_I9[1]]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["origin_fingerprint"], "deadbeef");
    assert_eq!(v["schema_version"], 1);
    let want = format!(
        "warning: this card is invalid ({I9_DISPLAY}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it\n"
    );
    assert!(s(&o.stderr).starts_with(&want), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_inspect_keeps_chunk_set_id_on_legacy_card() {
    let o = mk(&["inspect", LEGACY_I8[0], LEGACY_I8[1]]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        s(&o.stdout).contains("\nchunk_set_id:        ca522 (stamped)\n"),
        "stdout: {}",
        s(&o.stdout)
    );
}
