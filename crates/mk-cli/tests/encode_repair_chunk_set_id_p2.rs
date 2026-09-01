//! P2 golden tests: `mk encode --chunk-set-id` mint warning (SPEC contract
//! 5) and `mk repair`'s blessed-path warning (contract 2's `mk repair`
//! coverage, r4 L2-I1/I2). `repair --json` byte-match regression (D27).
//!
//! Realizes `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` P2.
//! P1 covers decode/inspect/verify/derive/address (`csid_verification.rs`);
//! this file covers the two verbs P1 explicitly deferred.
//!
//! The repair half is driven live off the P0 extension corpus's pinned
//! mismatch row (`SEED_pinned_12345_ef12f`, declared `12345` / derived
//! `ef12f`) via `mk_codec::test_vectors::csid_ext::CSID_EXT_JSON` -- same
//! sourcing discipline as P1's `csid_verification.rs`. The mint half has no
//! corpus row of its own (the corpus rows are already-minted cards), so it
//! derives its OWN "real" id live, per `encode_chunk_set_id.rs`'s
//! established pattern with the same `V1_XPUB` fixture (measured live this
//! session: derived `83bb2`; re-derived at test time rather than hardcoded
//! so a future mk-codec change can't leave a stale constant comparing
//! against itself).

use assert_cmd::Command;
use serde_json::Value;

// ============================================================================
// Shared corpus access (mirrors csid_verification.rs's helpers).
// ============================================================================

const MISMATCH_ROW: &str = "SEED_pinned_12345_ef12f";

fn corpus() -> Value {
    serde_json::from_str(mk_codec::test_vectors::csid_ext::CSID_EXT_JSON)
        .expect("parse csid_ext_v0.1.json")
}

fn row_strings(doc: &Value, name: &str) -> Vec<String> {
    let rows = doc["rows"].as_array().expect("rows is array");
    let row = rows
        .iter()
        .find(|r| r["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("corpus row {name:?} not found"));
    row["strings"]
        .as_array()
        .expect("row.strings is array")
        .iter()
        .map(|v| v.as_str().expect("string").to_string())
        .collect()
}

// ============================================================================
// Bech32 5-bit symbol helpers -- copied from cli_mk1_repair_reverify.rs
// (this file has no access to mk-cli's `pub(crate)` internals, only
// mk-codec's public `ALPHABET`).
// ============================================================================

fn char_to_5bit(c: char) -> u8 {
    mk_codec::string_layer::bch::ALPHABET
        .iter()
        .position(|&b| b as char == c)
        .expect("bech32 alphabet char") as u8
}

fn data_part_values(s: &str) -> (&str, Vec<u8>) {
    let sep = s.rfind('1').expect("bech32 separator");
    let (prefix, rest) = s.split_at(sep + 1);
    (prefix, rest.chars().map(char_to_5bit).collect())
}

fn rebuild_string(prefix: &str, values: &[u8]) -> String {
    let mut out = String::from(prefix);
    for &v in values {
        out.push(mk_codec::string_layer::bch::ALPHABET[v as usize] as char);
    }
    out
}

/// A single deterministic substitution at data-part position 20 -- past the
/// 8-symbol chunked header (so the damaged chunk stays in the SAME
/// `chunk_set_id` group; a header-region flip would risk changing which
/// group it lands in) and trivially within both codes' t<=4 correction
/// radius (measured live: both fixtures below correct cleanly back to the
/// original at this position).
fn flip_at(chunk: &str, pos: usize) -> String {
    let (prefix, mut values) = data_part_values(chunk);
    let was = values[pos];
    values[pos] = (was + 1) % 32;
    rebuild_string(prefix, &values)
}

// ============================================================================
// (a) Mint warning -- SPEC contract 5.
// ============================================================================

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";

fn encode(extra: &[&str]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.args([
        "encode",
        "--xpub",
        V1_XPUB,
        "--origin-fingerprint",
        "aabbccdd",
        "--origin-path",
        "m/48'/0'/0'/2'",
        "--policy-id-stub",
        "11223344",
        "--group-size",
        "0",
    ]);
    cmd.args(extra);
    cmd.output().unwrap()
}

/// Mint unpinned, then read the stamped id back via `mk inspect` (contract
/// 3's unconditional print) to learn this fixture's real content-derived
/// id.
fn derived_id_of_v1_fixture() -> String {
    let out = encode(&[]);
    assert!(
        out.status.success(),
        "unpinned mint must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let strings: Vec<&str> = stdout.lines().collect();
    assert!(strings.len() > 1, "this fixture must chunk or the test proves nothing");

    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.arg("inspect");
    cmd.args(&strings);
    let insp = cmd.output().unwrap();
    assert!(insp.status.success());
    let insp_stdout = String::from_utf8(insp.stdout).unwrap();
    let line = insp_stdout
        .lines()
        .find(|l| l.contains("chunk_set_id:"))
        .expect("inspect must print chunk_set_id (contract 3)");
    line.split_whitespace()
        .nth(1)
        .expect("chunk_set_id token")
        .to_string()
}

#[test]
fn mint_pinned_mismatch_warns_stderr_exit0_strings_still_mint() {
    let derived = derived_id_of_v1_fixture();
    assert_ne!(
        derived, "12345",
        "fixture must actually mismatch 0x12345 or this test proves nothing"
    );

    let out = encode(&["--chunk-set-id", "0x12345"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "mint must still succeed at exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        !stdout.trim().is_empty() && stdout.lines().all(|l| l.starts_with("mk1")),
        "the mk1 strings must still mint on stdout; stdout={stdout:?}"
    );

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("warning: --chunk-set-id pins 12345 in place of the content-derived id"),
        "stderr must carry the contract-5 mint warning; stderr={stderr:?}"
    );
    assert!(
        stderr.contains(&derived),
        "stderr must name the actual derived id ({derived}); stderr={stderr:?}"
    );
    assert!(
        stderr.contains("drop --chunk-set-id entirely"),
        "stderr must carry the drop-the-flag remedy; stderr={stderr:?}"
    );
    assert!(
        stderr.contains("Do not re-type the derived value into the flag"),
        "stderr must carry the anti-transcription clause (W14); stderr={stderr:?}"
    );
    assert!(
        stderr.contains("never engrave this on a real plate"),
        "stderr must carry the test-fixtures-only clause; stderr={stderr:?}"
    );
}

#[test]
fn mint_pin_equal_to_derived_id_is_silent() {
    let derived = derived_id_of_v1_fixture();
    let out = encode(&["--chunk-set-id", &derived]);
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !stderr.contains("--chunk-set-id pins"),
        "a pin EQUAL to the derived id is not a mismatch and must stay silent; stderr={stderr:?}"
    );
}

// ============================================================================
// (b) `mk repair` blessed-path warning -- SPEC contract 2's repair coverage
// (r4 L2-I1/I2) -- plus the `repair --json` byte-match regression (D27).
// ============================================================================

/// Pre-P2 golden `mk repair --json` output for the pinned-mismatch row's
/// chunk 0 (untouched) + a damaged chunk 1 (one substitution at data-part
/// position 20, 'g'->'f', correcting back to 'g'), captured LIVE against
/// the pre-P2 binary this session. Proves the envelope stays byte-for-byte
/// unchanged (D27 cross-CLI contract with `mnemonic repair --json`), not
/// merely schema-compatible.
const PRE_P2_REPAIR_JSON: &str = "{\"schema_version\":\"1\",\"kind\":\"mk1\",\"corrected_chunks\":[\"mk1qpzg69pqqsqsqrrhvket9v4jq5zg3vs7zqsrq9dlh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alhupawhtfl552clzu3rgv\",\"mk1qpzg69ppjd334aa2pecfgwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3\"],\"repairs\":[{\"chunk_index\":1,\"original_chunk\":\"mk1qpzg69ppjd334aa2pecffwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3\",\"corrected_chunk\":\"mk1qpzg69ppjd334aa2pecfgwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3\",\"corrected_positions\":[{\"position\":20,\"was\":\"f\",\"now\":\"g\"}]}]}";

#[test]
fn repair_blessed_damaged_pinned_card_warns_with_mint_time_clause() {
    let doc = corpus();
    let strings = row_strings(&doc, MISMATCH_ROW);
    assert_eq!(strings.len(), 2, "pinned mismatch row must be a 2-chunk card");
    let chunk0 = strings[0].clone();
    let chunk1 = strings[1].clone();
    let damaged_chunk1 = flip_at(&chunk1, 20);
    assert_ne!(damaged_chunk1, chunk1);

    let out = Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", &chunk0, &damaged_chunk1])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(5),
        "a genuine correction that reassembles must still exit 5; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(stdout.contains("# Repair report"), "stdout={stdout:?}");
    assert!(
        stdout.contains(&chunk1),
        "the corrected chunk must reproduce the original undamaged chunk; stdout={stdout:?}"
    );

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("this key card's stamped chunk-set id (12345) was not derived from"),
        "stderr must carry the R2 pair on the blessed re-verify path; stderr={stderr:?}"
    );
    assert!(
        stderr.contains("ef12f"),
        "stderr must name the content-derived id; stderr={stderr:?}"
    );
    assert!(
        stderr.contains("This id was set when the card was minted; the repair did not change it."),
        "stderr must carry the mint-time clause (r4 L2-I2), or the repair report reads as \
         \"I changed your card\"; stderr={stderr:?}"
    );
}

#[test]
fn repair_json_byte_unchanged_no_chunk_set_id_field() {
    let doc = corpus();
    let strings = row_strings(&doc, MISMATCH_ROW);
    let chunk0 = strings[0].clone();
    let chunk1 = strings[1].clone();
    let damaged_chunk1 = flip_at(&chunk1, 20);

    let out = Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", "--json", &chunk0, &damaged_chunk1])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(5));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout.trim_end(),
        PRE_P2_REPAIR_JSON,
        "repair --json must stay byte-identical to pre-P2 (D27 cross-CLI contract) -- the \
         mismatch warning belongs on stderr, never in this envelope"
    );
    assert!(
        !stdout.contains("chunk_set_id"),
        "repair --json must gain no chunk_set_id field this cycle (r4 L2-I3); stdout={stdout:?}"
    );

    // The warning still fires on stderr even in --json mode: contract 2 is
    // a stderr-only advisory, and --json is the ENVELOPE that stays
    // unchanged, not the whole invocation's output.
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("This id was set when the card was minted"),
        "stderr={stderr:?}"
    );
}

#[test]
fn repair_undamaged_pinned_card_exit0_silent() {
    let doc = corpus();
    let strings = row_strings(&doc, MISMATCH_ROW);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", &strings[0], &strings[1]])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "an already-valid supply applies no correction and never reaches the classifier"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !stderr.contains("was not derived from its content") && !stderr.contains("chunk-set id"),
        "an already-valid supply never decodes on repair's blessed path and must stay silent \
         (r4 L2-I1); stderr={stderr:?}"
    );
}

#[test]
fn repair_single_chunk_candidate_of_pinned_card_silent() {
    let doc = corpus();
    let strings = row_strings(&doc, MISMATCH_ROW);
    let damaged_chunk0 = flip_at(&strings[0], 20);
    let out = Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", &damaged_chunk0])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(5),
        "a single corrected chunk supplied alone still exits 5 (Candidate)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("UNVERIFIED"),
        "the existing Candidate advisory must be unaffected; stderr={stderr:?}"
    );
    assert!(
        !stderr.contains("was not derived from its content") && !stderr.contains("chunk-set id"),
        "a Candidate group is incomplete and never decodes, so it must never reach the R2 \
         warning (r4 L2-I1); stderr={stderr:?}"
    );
}
