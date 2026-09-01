//! P1 golden tests: the R2 mismatch warning on the FIVE read-side mk-cli
//! verbs (decode/inspect/verify/derive/address), driven live by the P0
//! extension corpus (`mk_codec::test_vectors::csid_ext::CSID_EXT_JSON`).
//!
//! Realizes `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` P1 and
//! `design/SPEC_chunk_set_id_verification.md` contracts 2-4. `mk repair`
//! (contract 2's sixth verb) is P2 and is NOT covered here.
//!
//! Fixture rows (read live from the corpus, never hand-transcribed, so a
//! corpus regeneration that changes wording or ids cannot silently desync
//! from this test):
//!
//! - `SEED_pinned_12345_ef12f` -- THE pinned mismatch row this warning
//!   exists for: declared `12345`, content-derived `ef12f`.
//! - `SEED_plate_b_ef12f` -- its clean twin: byte-identical content, but
//!   minted without `--chunk-set-id`, so declared == derived == `ef12f`.
//! - `LZ1_derived_below_0x10000` -- a clean row whose id is `< 0x10000`
//!   (`0191c`), exercising the `{:05x}` leading-zero rendering path.
//!
//! Both seed cards' origin path is `48'/0'/0'/2'` (BIP-48 multisig
//! cosigner), so `mk address` refuses them with exit 64 regardless of R2 --
//! the warning still fires at `address`'s own decode call, BEFORE that
//! unrelated refusal, which is exactly what these tests assert.

use assert_cmd::Command;
use serde_json::{Value, json};

const MISMATCH_ROW: &str = "SEED_pinned_12345_ef12f";
const CLEAN_ROW: &str = "SEED_plate_b_ef12f";
const LZ_ROW: &str = "LZ1_derived_below_0x10000";

/// Parse the P0 extension corpus.
fn corpus() -> Value {
    serde_json::from_str(mk_codec::test_vectors::csid_ext::CSID_EXT_JSON)
        .expect("parse csid_ext_v0.1.json")
}

/// The named row's mk1 strings.
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

/// The pinned row's frozen `warning_text` (spec contract 2 / R6), read live
/// rather than hand-copied.
fn pinned_warning_text(doc: &Value) -> String {
    let rows = doc["rows"].as_array().expect("rows is array");
    let row = rows
        .iter()
        .find(|r| r["name"].as_str() == Some(MISMATCH_ROW))
        .expect("pinned row present");
    let text = row["warning_text"]
        .as_str()
        .expect("warning_text is string")
        .to_string();
    assert!(!text.is_empty(), "pinned row must carry a non-empty warning_text");
    text
}

fn run(verb: &str, extra: &[&str], strings: &[String]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.arg(verb);
    cmd.args(extra);
    for s in strings {
        cmd.arg(s);
    }
    cmd.output().expect("invoke mk")
}

/// Shared assertion for the five read-side verbs: `verb` against the
/// PINNED MISMATCH row warns on stderr with the exact pinned content
/// (declared/derived pair + remedy sentence); against its CLEAN TWIN it is
/// silent; the exit code is IDENTICAL between the two runs (R2 is a
/// non-fatal advisory and must never change what a verb returns).
///
/// `expect_exit_ok`: whether the verb is expected to succeed (exit 0) on
/// this fixture's shape. `false` only for `address`, which refuses this
/// BIP-48 multisig-cosigner fixture regardless of R2.
fn assert_warns_on_mismatch_silent_on_clean(verb: &str, extra: &[&str], expect_exit_ok: bool) {
    let doc = corpus();
    let warning = pinned_warning_text(&doc);
    let mismatch = row_strings(&doc, MISMATCH_ROW);
    let clean = row_strings(&doc, CLEAN_ROW);

    let m = run(verb, extra, &mismatch);
    let m_stderr = String::from_utf8_lossy(&m.stderr).to_string();
    assert!(
        m_stderr.contains(&warning),
        "{verb}: mismatch row must warn verbatim on stderr (matches corpus warning_text); \
         stderr={m_stderr:?}"
    );
    assert!(
        m_stderr.contains("12345") && m_stderr.contains("ef12f"),
        "{verb}: the (declared, derived) pair must appear; stderr={m_stderr:?}"
    );
    assert!(
        m_stderr.contains("re-mint") && m_stderr.contains("mk encode again without --chunk-set-id"),
        "{verb}: the remedy sentence must appear; stderr={m_stderr:?}"
    );
    if expect_exit_ok {
        assert_eq!(
            m.status.code(),
            Some(0),
            "{verb}: mismatch row must still succeed at exit 0 (the warning is non-fatal); \
             stderr={m_stderr}"
        );
    }

    let c = run(verb, extra, &clean);
    let c_stderr = String::from_utf8_lossy(&c.stderr).to_string();
    assert!(
        !c_stderr.contains("was not derived from its content"),
        "{verb}: clean twin must be SILENT on the R2 warning; stderr={c_stderr:?}"
    );

    assert_eq!(
        m.status.code(),
        c.status.code(),
        "{verb}: R2 must not change the exit code (mismatch row vs. its clean twin)"
    );
}

#[test]
fn decode_mismatch_row_warns_clean_twin_silent() {
    assert_warns_on_mismatch_silent_on_clean("decode", &[], true);
}

#[test]
fn inspect_mismatch_row_warns_clean_twin_silent() {
    assert_warns_on_mismatch_silent_on_clean("inspect", &[], true);
}

#[test]
fn verify_mismatch_row_warns_clean_twin_silent() {
    assert_warns_on_mismatch_silent_on_clean("verify", &[], true);
}

#[test]
fn derive_mismatch_row_warns_clean_twin_silent() {
    assert_warns_on_mismatch_silent_on_clean("derive", &["--index", "0"], true);
}

#[test]
fn address_mismatch_row_warns_clean_twin_silent() {
    assert_warns_on_mismatch_silent_on_clean("address", &["--count", "1"], false);
}

/// Contract 4: `mk verify` (text mode) carries the mismatch on its OWN
/// STDOUT verdict, not only on stderr -- a consumer reading only stdout
/// must still see it.
#[test]
fn verify_stdout_verdict_carries_the_mismatch() {
    let doc = corpus();
    let warning = pinned_warning_text(&doc);
    let mismatch = row_strings(&doc, MISMATCH_ROW);
    let clean = row_strings(&doc, CLEAN_ROW);

    let m = run("verify", &[], &mismatch);
    assert_eq!(m.status.code(), Some(0));
    let m_stdout = String::from_utf8_lossy(&m.stdout).to_string();
    assert!(
        m_stdout.starts_with("OK:"),
        "verify's own OK verdict must still lead; stdout={m_stdout:?}"
    );
    assert!(
        m_stdout.contains(&warning),
        "verify: mismatch must carry the pair + remedy on STDOUT too (contract 4), not only \
         stderr; stdout={m_stdout:?}"
    );

    let c = run("verify", &[], &clean);
    assert_eq!(c.status.code(), Some(0));
    let c_stdout = String::from_utf8_lossy(&c.stdout).to_string();
    assert!(
        !c_stdout.contains("was not derived from its content"),
        "verify: clean twin stdout must be silent; stdout={c_stdout:?}"
    );
}

/// Contract 4: `mk verify --json` gains the additive `chunk_set_id` object
/// -- present (with `matches`) for chunked input on BOTH a mismatch and a
/// clean row; `schema_version` stays the integer `1`.
#[test]
fn verify_json_chunk_set_id_object() {
    let doc = corpus();
    let mismatch = row_strings(&doc, MISMATCH_ROW);
    let clean = row_strings(&doc, CLEAN_ROW);

    let m = run("verify", &["--json"], &mismatch);
    assert_eq!(m.status.code(), Some(0));
    let m_stdout = String::from_utf8(m.stdout).unwrap();
    let v_m: Value = serde_json::from_str(m_stdout.trim()).expect("verify --json parses");
    assert!(
        v_m["schema_version"].is_number(),
        "schema_version must stay an INTEGER (not the string \"1\" `mk repair --json` uses); \
         got {:?}",
        v_m["schema_version"]
    );
    assert_eq!(v_m["schema_version"], json!(1));
    assert_eq!(v_m["chunk_set_id"]["declared"], json!("12345"));
    assert_eq!(v_m["chunk_set_id"]["derived"], json!("ef12f"));
    assert_eq!(v_m["chunk_set_id"]["matches"], json!(false));

    let c = run("verify", &["--json"], &clean);
    assert_eq!(c.status.code(), Some(0));
    let c_stdout = String::from_utf8(c.stdout).unwrap();
    let v_c: Value = serde_json::from_str(c_stdout.trim()).expect("verify --json parses");
    assert_eq!(v_c["schema_version"], json!(1));
    assert_eq!(v_c["chunk_set_id"]["declared"], json!("ef12f"));
    assert_eq!(v_c["chunk_set_id"]["derived"], json!("ef12f"));
    assert_eq!(v_c["chunk_set_id"]["matches"], json!(true));
}

/// Contract 3: `mk inspect` prints the stamped `chunk_set_id`
/// UNCONDITIONALLY -- matched or not -- in text mode; and renders it with
/// the `{:05x}` leading-zero rule (`LZ1`'s id is `< 0x10000`).
#[test]
fn inspect_prints_stamped_chunk_set_id_unconditionally() {
    let doc = corpus();
    let mismatch = row_strings(&doc, MISMATCH_ROW);
    let clean = row_strings(&doc, CLEAN_ROW);
    let lz = row_strings(&doc, LZ_ROW);

    let out_m = run("inspect", &[], &mismatch);
    assert_eq!(out_m.status.code(), Some(0));
    let stdout_m = String::from_utf8_lossy(&out_m.stdout).to_string();
    assert!(
        stdout_m.contains("chunk_set_id:") && stdout_m.contains("12345"),
        "inspect must print the STAMPED id even on a mismatch; stdout={stdout_m:?}"
    );

    let out_c = run("inspect", &[], &clean);
    assert_eq!(out_c.status.code(), Some(0));
    let stdout_c = String::from_utf8_lossy(&out_c.stdout).to_string();
    assert!(
        stdout_c.contains("chunk_set_id:") && stdout_c.contains("ef12f"),
        "inspect must print the stamped id on a clean (matched) row too; stdout={stdout_c:?}"
    );

    let out_lz = run("inspect", &[], &lz);
    assert_eq!(out_lz.status.code(), Some(0));
    let stdout_lz = String::from_utf8_lossy(&out_lz.stdout).to_string();
    assert!(
        stdout_lz.contains("chunk_set_id:        0191c"),
        "inspect must render the leading zero ({{:05x}}), not truncate to 4 digits; \
         stdout={stdout_lz:?}"
    );
}
