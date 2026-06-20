//! Form-aware `policy_id_stub` derivation tests (`feature/mk-cli-template-id-stub`).
//!
//! `derive_stub_from_md1` must root the 4-byte stub on the SAME identity the
//! `mnemonic-toolkit` does for a given md1 *form* (toolkit #28,
//! `bundle --md1-form=template`):
//!
//! - a **keyless template** md1 (`!is_wallet_policy()`) → top 4 bytes of
//!   `md_codec::compute_wallet_descriptor_template_id` (BIP-388 template-only
//!   identity — key-stable);
//! - a **keyed wallet-policy** md1 (`is_wallet_policy()`) → top 4 bytes of
//!   `md_codec::compute_wallet_policy_id` (canonical-expanded policy identity,
//!   the pre-#28 behavior, preserved).
//!
//! INDEPENDENT goldens (audit I1, 2026-06-10 discipline): each `EXPECTED_*` is
//! a frozen literal computed ONCE out-of-band against md-codec 0.37.0, NOT a
//! runtime recomputation of the impl's own chain. The test body MUST NOT call
//! `compute_wallet_descriptor_template_id` / `compute_wallet_policy_id` — that
//! is what lets these cells catch a future re-divergence of the form dispatch.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V1_FP_HEX: &str = "aabbccdd";
const V1_PATH: &str = "m/48'/0'/0'/2'";

/// Canonical KEYLESS template md1 — md-codec's `pkh_basic` vector
/// (`tests/vectors/pkh_basic.phrase.txt`). `is_wallet_policy() == false`
/// (a plain `pkh(@0/**)` template carries no `Pubkeys` TLV).
const PKH_BASIC_TEMPLATE_MD1: &str = "md1yqpqqxzq2qwfv8urt848e";

/// Top 4 bytes of `compute_wallet_descriptor_template_id(PKH_BASIC_TEMPLATE_MD1)`
/// — the CORRECT, form-aware stub for a keyless template (post-#28).
const EXPECTED_TEMPLATE_STUB: [u8; 4] = [0x55, 0x9e, 0x64, 0xb2];

/// Top 4 bytes of `compute_wallet_policy_id(PKH_BASIC_TEMPLATE_MD1)` — the
/// pre-#28 (BUGGY for a template) value. Asserting the emitted stub does NOT
/// equal this guards against a regression to the unconditional policy-id path.
const POLICY_STUB_FOR_TEMPLATE: [u8; 4] = [0x3d, 0x19, 0x0a, 0xf3];

/// A synthetic KEYED wallet-policy md1: the `pkh_basic` template with a single
/// valid `@0` xpub + fingerprint injected into the `Pubkeys`/`Fingerprints`
/// TLVs, so `is_wallet_policy() == true`. Minted out-of-band against md-codec
/// 0.37.0 (chain-code = 0x42*32 || compressed pubkey `0279be66...f81798`).
const KEYED_WALLET_POLICY_MD1: &str = "md1yqpqqxzqksdatd7au25zzzgfpyysjzgfpyysjzgfpyysjzgfpyysjzgfpyysjzgfpyysjzggp8n0nx0muaewav2ksx99wwsu9swq5mlndjmn3gm9vl9q2mzmup0xqvgudy3j07cxwl";

/// Top 4 bytes of `compute_wallet_policy_id(KEYED_WALLET_POLICY_MD1)` — the
/// CORRECT stub for a keyed wallet-policy md1 (the pre-#28 path, preserved).
const EXPECTED_KEYED_POLICY_STUB: [u8; 4] = [0x2a, 0xed, 0x98, 0x0d];

/// `mk encode --from-md1 <keyless template>` must stamp the mk1 card with the
/// `WalletDescriptorTemplateId`-derived stub, NOT the `WalletPolicyId` one.
#[test]
fn encode_from_keyless_template_md1_uses_template_id_stub() {
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
            PKH_BASIC_TEMPLATE_MD1,
            // keep stdout lines unbroken — they feed mk_codec::decode directly.
            "--group-size",
            "0",
        ])
        .output()
        .expect("invoke mk encode");
    assert!(out.status.success(), "mk encode failed: {out:?}");

    let stdout = String::from_utf8(out.stdout).unwrap();
    let strings: Vec<String> = stdout.lines().map(str::to_string).collect();
    assert!(!strings.is_empty(), "no mk1 strings on stdout");

    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs).expect("decode emitted strings");

    assert_eq!(card.policy_id_stubs.len(), 1);
    assert_eq!(
        card.policy_id_stubs[0], EXPECTED_TEMPLATE_STUB,
        "keyless template md1 must use the WalletDescriptorTemplateId stub"
    );
    assert_ne!(
        card.policy_id_stubs[0], POLICY_STUB_FOR_TEMPLATE,
        "must NOT fall back to the unconditional WalletPolicyId stub for a template"
    );
}

/// `mk verify --from-md1 <keyless template>` must derive the same template-id
/// stub, so a card stamped with it verifies OK (exit 0) and one stamped with
/// the old policy-id stub does NOT.
#[test]
fn verify_from_keyless_template_md1_matches_template_id_stub() {
    // 1. Build a card carrying the CORRECT template-id stub via `mk encode`.
    let mut enc = Command::cargo_bin("mk").expect("mk binary");
    let out = enc
        .args([
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-fingerprint",
            V1_FP_HEX,
            "--origin-path",
            V1_PATH,
            "--from-md1",
            PKH_BASIC_TEMPLATE_MD1,
            "--group-size",
            "0",
        ])
        .output()
        .expect("invoke mk encode");
    assert!(out.status.success(), "mk encode failed: {out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let strings: Vec<String> = stdout.lines().map(str::to_string).collect();
    assert!(!strings.is_empty());

    // 2. `mk verify --from-md1 <same template>` must agree → exit 0.
    let mut ver = Command::cargo_bin("mk").expect("mk binary");
    let mut args: Vec<String> = vec!["verify".into()];
    args.extend(strings.iter().cloned());
    args.push("--from-md1".into());
    args.push(PKH_BASIC_TEMPLATE_MD1.into());
    let vout = ver.args(&args).output().expect("invoke mk verify");
    assert!(
        vout.status.success(),
        "mk verify --from-md1 (template) must exit 0; stderr={}",
        String::from_utf8_lossy(&vout.stderr)
    );

    // 3. A card explicitly stamped with the OLD policy-id stub must FAIL the
    //    same `--from-md1` verify (exit 4, ContentMismatch) — the two stubs
    //    differ, so the form-aware derivation is genuinely exercised.
    let mut enc2 = Command::cargo_bin("mk").expect("mk binary");
    let out2 = enc2
        .args([
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-fingerprint",
            V1_FP_HEX,
            "--origin-path",
            V1_PATH,
            "--policy-id-stub",
            "3d190af3",
            "--group-size",
            "0",
        ])
        .output()
        .expect("invoke mk encode (policy-stub card)");
    assert!(
        out2.status.success(),
        "mk encode (policy-stub) failed: {out2:?}"
    );
    let stdout2 = String::from_utf8(out2.stdout).unwrap();
    let strings2: Vec<String> = stdout2.lines().map(str::to_string).collect();

    let mut ver2 = Command::cargo_bin("mk").expect("mk binary");
    let mut args2: Vec<String> = vec!["verify".into()];
    args2.extend(strings2.iter().cloned());
    args2.push("--from-md1".into());
    args2.push(PKH_BASIC_TEMPLATE_MD1.into());
    let vout2 = ver2
        .args(&args2)
        .output()
        .expect("invoke mk verify (mismatch)");
    assert_eq!(
        vout2.status.code(),
        Some(4),
        "policy-id-stamped card must MISMATCH the template-id derivation; stderr={}",
        String::from_utf8_lossy(&vout2.stderr)
    );
}

/// Regression: a KEYED wallet-policy md1 (`is_wallet_policy()`) must STILL use
/// the `WalletPolicyId` stub (the pre-#28 path is preserved by the new branch).
#[test]
fn encode_from_keyed_wallet_policy_md1_uses_policy_id_stub() {
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
            KEYED_WALLET_POLICY_MD1,
            "--group-size",
            "0",
        ])
        .output()
        .expect("invoke mk encode");
    assert!(out.status.success(), "mk encode failed: {out:?}");

    let stdout = String::from_utf8(out.stdout).unwrap();
    let strings: Vec<String> = stdout.lines().map(str::to_string).collect();
    assert!(!strings.is_empty());

    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs).expect("decode emitted strings");

    assert_eq!(card.policy_id_stubs.len(), 1);
    assert_eq!(
        card.policy_id_stubs[0], EXPECTED_KEYED_POLICY_STUB,
        "keyed wallet-policy md1 must keep using the WalletPolicyId stub"
    );
    assert_ne!(
        card.policy_id_stubs[0], EXPECTED_TEMPLATE_STUB,
        "keyed md1 must NOT switch to the template-id stub"
    );
}
