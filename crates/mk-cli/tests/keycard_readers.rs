//! Architect rows (bh #17 Q1, Q2): mk verify / decode on a legacy depth-1 card;
//! the T5 warning in inspect, address and derive; mk encode depth-1 guard.
use assert_cmd::cargo::CommandCargoExt;
use std::process::{Command, Output};

const LEGACY_D1: [&str; 2] = [
    "mk1qpkg6wpqqsqaatd7al02m0h0lcqsqpp4sl8h28nkaq2rw39uktvh6uenflm3rxqw9jllmjktc059yn75stew08xve94t2q3eze2hn68kjszz",
    "mk1qpkg6wppqgx00lerqaq42pmr6f79l0erqcejj2g8umwpudmkpn4h093lwzzehv35azeqll55lad4xzcy4",
];
const D1_TPUB: &str = "tpubD984EkAGpvBEL17UyUznoEtTRFfVULe1D6QeTARexMtG9Xvjsqo3cV2EuNcnQmAoodvTVoiLC1W8k4tF8zmrPwM8EuEK2KyvuhnYhYurri4";
const LEGACY_I8: [&str; 2] = [
    "mk1qpeffzpqqqqaatd7allqqpp4sl8szqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqnehen8u5yagl2mw0p43h3j",
    "mk1qpeffzppl8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqte3jjj9mtsugd0x98xyvntv",
];
const SENTINEL_ZERO: [&str; 2] = [
    "mk1qp2enrpqqsqaatd7auqqqqqqlcqqgdv8euqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy8h8vagvc68yhng5",
    "mk1qp2enrpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v9ve3hu53hjkuceu46e8y",
];
const D1_DISPLAY: &str = "origin fingerprint deadbeef is not the depth-1 key's parent fingerprint 751e76e8; at depth 1 the parent is the master";
const I8_DISPLAY: &str =
    "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key";

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
fn t5(e: &str) -> String {
    format!(
        "warning: this card is invalid ({e}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it\n"
    )
}
fn warns(verb: &[&str], card: [&str; 2], e: &str) {
    let mut a = verb.to_vec();
    a.extend_from_slice(&card);
    let o = mk(&a);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", s(&o.stderr));
    assert!(s(&o.stderr).contains(&t5(e)), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_verify_reports_invalid_depth1() {
    let o = mk(&["verify", LEGACY_D1[0], LEGACY_D1[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(s(&o.stderr), format!("error: {D1_DISPLAY}\n"));
}

#[test]
fn kv_cli_verify_json_kind_depth1() {
    let o = mk(&["verify", "--json", LEGACY_D1[0], LEGACY_D1[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "OriginParentFingerprintMismatch");
}

#[test]
fn kv_cli_decode_warns_depth1() {
    warns(&["decode"], LEGACY_D1, D1_DISPLAY);
}

#[test]
fn kv_cli_inspect_warns_i8() {
    warns(&["inspect"], LEGACY_I8, I8_DISPLAY);
}

#[test]
fn kv_cli_address_warns_i8() {
    warns(
        &["address", "--count", "1", "--address-type", "p2wpkh"],
        LEGACY_I8,
        I8_DISPLAY,
    );
}

#[test]
fn kv_cli_derive_warns_i8() {
    warns(&["derive", "--index", "0"], LEGACY_I8, I8_DISPLAY);
}

#[test]
fn kv_cli_derive_json_warns_depth1_stdout_json() {
    let o = mk(&[
        "derive",
        "--json",
        "--index",
        "0",
        LEGACY_D1[0],
        LEGACY_D1[1],
    ]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["parent_xpub"], D1_TPUB);
    assert!(
        s(&o.stderr).contains(&t5(D1_DISPLAY)),
        "stderr: {}",
        s(&o.stderr)
    );
}

#[test]
fn kv_cli_readers_sentinel_no_warning() {
    for verb in [
        &["inspect"][..],
        &["address", "--count", "1", "--address-type", "p2wpkh"][..],
        &["derive", "--index", "0"][..],
    ] {
        let mut a = verb.to_vec();
        a.extend_from_slice(&SENTINEL_ZERO);
        let o = mk(&a);
        assert_eq!(
            o.status.code(),
            Some(0),
            "{verb:?} stderr: {}",
            s(&o.stderr)
        );
        assert!(
            !s(&o.stderr).contains("this card is invalid"),
            "{verb:?} stderr: {}",
            s(&o.stderr)
        );
    }
}

#[test]
fn kv_cli_encode_depth1_false_fp_refused_guard() {
    let o = mk(&[
        "encode",
        "--xpub",
        D1_TPUB,
        "--origin-path",
        "m/0",
        "--origin-fingerprint",
        "deadbeef",
        "--policy-id-stub",
        "deadbeef",
    ]);
    assert_eq!(o.status.code(), Some(64), "stdout: {}", s(&o.stdout));
}
