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

/// A REAL keyed wallet-policy md1: `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` with
/// two concrete xpubs and a shared `m/48'/0'/0'/2'` origin, minted by the
/// primary `md encode --force-chunked`.
///
/// It arrives as FOUR chunks and cannot arrive any other way. A keyed policy
/// is 246 data symbols and the codex32 regular code caps a SINGLE md1 string
/// at 80, so every keyed wallet-policy card in the constellation is chunked.
/// The fixture this replaced was a hand-minted 138-symbol single string --
/// a card no encoder emits and no engraver could cut, which is why the gap it
/// was standing in for went unseen (F-127).
const KEYED_POLICY_A_CHUNKS: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];

/// A SECOND, distinct keyed wallet-policy: the same two keys at threshold 1
/// instead of 2. Used to prove `--from-md1` still means "one card per policy"
/// after chunk grouping -- eight strings here are TWO cards, not eight.
const KEYED_POLICY_B_CHUNKS: [&str; 4] = [
    "md1f8h9cpspq2tvyyy4qqxppsq2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapggcqjxy8deuaca",
    "md1f8h9cpsf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2w445dju4r5jkq8",
    "md1f8h9cpshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuz9lsyct426why",
    "md1f8h9cpslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqtjjhsvxyyzfma",
];

/// Top 4 bytes of the `WalletPolicyId` of [`KEYED_POLICY_A_CHUNKS`].
///
/// INDEPENDENT golden, cross-LANGUAGE: `md inspect` (Rust, primary) and the
/// seedhammer fork's `md.FormAwareIdChunks` (Go) were each asked for this card
/// and both answered `38bd7cec8059cdc4553c2836d7e8e303`, kind Policy-ID. Two
/// implementations in two languages agreeing is a stronger golden than one
/// out-of-band computation, and it is the check that found F-212 -- where Go
/// and Rust had silently disagreed on exactly this identity.
const EXPECTED_KEYED_POLICY_A_STUB: [u8; 4] = [0x38, 0xbd, 0x7c, 0xec];

/// Top 4 bytes of the `WalletPolicyId` of [`KEYED_POLICY_B_CHUNKS`].
/// Same cross-language derivation: `302f24a139b72a8e4fb04343168614f3`.
const EXPECTED_KEYED_POLICY_B_STUB: [u8; 4] = [0x30, 0x2f, 0x24, 0xa1];

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

/// Regression: a KEYED wallet-policy md1 must STILL use the `WalletPolicyId`
/// stub (the pre-#28 path is preserved by the form-aware branch) -- and it must
/// do so when the card arrives CHUNKED, which is the only way it ever arrives.
#[test]
fn encode_from_keyed_chunk_set_uses_policy_id_stub() {
    let stubs = stubs_from_md1(&KEYED_POLICY_A_CHUNKS);
    assert_eq!(
        stubs.len(),
        1,
        "four chunks are ONE card and must yield ONE stub, got {stubs:?}"
    );
    assert_eq!(
        stubs[0], EXPECTED_KEYED_POLICY_A_STUB,
        "keyed wallet-policy md1 must use the WalletPolicyId stub"
    );
    assert_ne!(
        stubs[0], EXPECTED_TEMPLATE_STUB,
        "keyed md1 must NOT switch to the template-id stub"
    );
}

/// `--from-md1` stays "one value per POLICY" after chunk grouping: eight
/// strings spanning two chunk sets are TWO cards, and their stubs land in
/// first-appearance order.
///
/// This is the assertion that keeps the grouping honest. A fix that simply
/// reassembled every `--from-md1` value into one descriptor would pass the
/// single-card test above and silently destroy the multi-policy case -- a key
/// card that belongs to two wallets would be stamped with one stub.
#[test]
fn two_chunk_sets_are_two_cards_in_order() {
    let mut all: Vec<&str> = KEYED_POLICY_A_CHUNKS.to_vec();
    all.extend_from_slice(&KEYED_POLICY_B_CHUNKS);
    let stubs = stubs_from_md1(&all);
    assert_eq!(
        stubs,
        vec![EXPECTED_KEYED_POLICY_A_STUB, EXPECTED_KEYED_POLICY_B_STUB],
        "two chunk sets must yield two stubs in first-appearance order"
    );
}

/// Chunks of one set may be INTERLEAVED with another set's and still group
/// correctly: grouping keys on the 20-bit chunk-set-id in each wire header,
/// not on adjacency. Order of the RESULT still follows first appearance.
#[test]
fn interleaved_chunk_sets_still_group_by_set_id() {
    let mut all: Vec<&str> = Vec::new();
    for i in 0..4 {
        all.push(KEYED_POLICY_A_CHUNKS[i]);
        all.push(KEYED_POLICY_B_CHUNKS[i]);
    }
    let stubs = stubs_from_md1(&all);
    assert_eq!(
        stubs,
        vec![EXPECTED_KEYED_POLICY_A_STUB, EXPECTED_KEYED_POLICY_B_STUB],
        "interleaved chunks must group by set-id, not adjacency"
    );
}

/// A single-string card and a chunk set may be mixed in one invocation.
#[test]
fn single_string_and_chunk_set_mix() {
    let mut all: Vec<&str> = vec![PKH_BASIC_TEMPLATE_MD1];
    all.extend_from_slice(&KEYED_POLICY_A_CHUNKS);
    let stubs = stubs_from_md1(&all);
    assert_eq!(
        stubs,
        vec![EXPECTED_TEMPLATE_STUB, EXPECTED_KEYED_POLICY_A_STUB],
        "a keyless single string and a keyed chunk set are two cards"
    );
}

/// An INCOMPLETE chunk set must be refused, not silently stubbed from the
/// chunks that happened to be present. Reassembly is what enforces this; the
/// test pins that the CLI does not swallow the codec's refusal.
#[test]
fn incomplete_chunk_set_is_refused() {
    let out = encode_with_md1(&KEYED_POLICY_A_CHUNKS[..3]);
    assert!(
        !out.status.success(),
        "three of four chunks must not produce a card"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    // "got 3" and not "got 1": the three chunks must have been GROUPED and
    // offered to reassembly together. Asserting only "chunk set incomplete"
    // would pass on the pre-fix code too, where each chunk was decoded alone
    // and refused as a set of one -- a refusal that is right by accident.
    assert!(
        stderr.contains("chunk set incomplete: got 3 chunks, expected 4"),
        "refusal must show the three chunks were grouped; got: {stderr}"
    );
}

/// Run `mk encode` with one `--from-md1` per value in `md1s`.
fn encode_with_md1(md1s: &[&str]) -> std::process::Output {
    let mut args: Vec<String> = vec![
        "encode".into(),
        "--xpub".into(),
        V1_XPUB.into(),
        "--origin-fingerprint".into(),
        V1_FP_HEX.into(),
        "--origin-path".into(),
        V1_PATH.into(),
        "--group-size".into(),
        "0".into(),
    ];
    for m in md1s {
        args.push("--from-md1".into());
        args.push((*m).into());
    }
    Command::cargo_bin("mk")
        .expect("mk binary")
        .args(&args)
        .output()
        .expect("invoke mk encode")
}

/// The stubs `mk encode --from-md1 <md1s...>` stamps onto the emitted card,
/// read back by DECODING the mk1 output rather than by trusting stdout text.
fn stubs_from_md1(md1s: &[&str]) -> Vec<[u8; 4]> {
    let out = encode_with_md1(md1s);
    assert!(
        out.status.success(),
        "mk encode failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let strings: Vec<String> = stdout.lines().map(str::to_string).collect();
    assert!(!strings.is_empty(), "no mk1 strings on stdout");
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    mk_codec::decode(&refs)
        .expect("decode emitted strings")
        .policy_id_stubs
}
