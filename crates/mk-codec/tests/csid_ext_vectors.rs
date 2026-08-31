//! `csid_ext_v0.1.json` integration tests (Phase P0 of
//! `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md`).
//!
//! This is the NEW extension corpus's reader test — a sibling of
//! `tests/vectors.rs` (the legacy v0.1.json harness), which stays
//! untouched: the legacy corpus is the pinned-by-design MISMATCH half
//! and this file supplies the CLEAN half plus warning content.
//!
//! For every row it asserts:
//!
//! 1. `derived_csid` reproduces `derive_chunk_set_id(encode_bytecode(decode(strings)))`,
//!    recomputed live from the row's own `strings` — never trusted as data.
//! 2. `canonical_bytecode_hex` reproduces `encode_bytecode(decode(strings))`.
//! 3. `expect_mismatch_warning == (declared_csid != derived_csid)`.
//! 4. The on-disk JSON's SHA-256 matches the pinned `CSID_EXT_SHA256`
//!    constant — a SEPARATE pin from `tests/vectors.rs::V0_1_SHA256`.
//!
//! It also asserts the extension corpus carries one row per entry of
//! `mk_codec::bytecode::STANDARD_PATHS` (`SP##_std_path_0x..` rows), so a
//! future 15th table entry trips THIS test rather than silently going
//! uncovered (spec r4 L1-I1).
//!
//! `v0.1.json`'s own byte-unchanged pin lives in
//! `tests/vectors.rs::vector_file_sha256_matches_pin` and is unaffected by
//! this file — confirmed green alongside this suite, not re-asserted here
//! (duplicating a pin risks the two drifting out of sync).

use mk_codec::bytecode::{STANDARD_PATHS, encode_bytecode};
use mk_codec::test_vectors::csid_ext::CSID_EXT_JSON;
use mk_codec::{decode, derive_chunk_set_id};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Pinned SHA-256 of `crates/mk-codec/src/test_vectors/csid_ext_v0.1.json`.
///
/// Update via:
///
/// ```text
/// cargo run --bin gen_mk_vectors --features gen-vectors
/// sha256sum crates/mk-codec/src/test_vectors/csid_ext_v0.1.json
/// # paste the hex into CSID_EXT_SHA256
/// ```
///
/// SEPARATE from `tests/vectors.rs::V0_1_SHA256` — the legacy corpus is
/// untouched by this cycle (plan P0).
const CSID_EXT_SHA256: &str = "88bbe056e85dde694353475e774a78a00defe75cb8694654c4be1d2467ad68f9";

fn doc() -> Value {
    serde_json::from_str(CSID_EXT_JSON).expect("parse csid_ext_v0.1.json")
}

fn rows(doc: &Value) -> &Vec<Value> {
    doc["rows"].as_array().expect("rows is array")
}

#[test]
fn csid_ext_sha256_matches_pin() {
    let digest = Sha256::digest(CSID_EXT_JSON.as_bytes());
    let actual = hex::encode(digest);
    assert_eq!(
        actual, CSID_EXT_SHA256,
        "src/test_vectors/csid_ext_v0.1.json SHA-256 drifted; if intended, \
         regenerate via `cargo run --bin gen_mk_vectors --features gen-vectors` \
         and update `CSID_EXT_SHA256` in tests/csid_ext_vectors.rs"
    );
}

#[test]
fn corpus_is_nonempty_and_every_row_recomputes_live() {
    let doc = doc();
    let rows = rows(&doc);
    assert!(
        !rows.is_empty(),
        "csid_ext corpus must not be empty (P0: it must exist and carry rows \
         before any downstream surface asserts against it)"
    );

    for row in rows {
        let name = row["name"].as_str().expect("row.name is string").to_string();

        let strings: Vec<String> = row["strings"]
            .as_array()
            .unwrap_or_else(|| panic!("[{name}] row.strings is array"))
            .iter()
            .map(|v| v.as_str().expect("string").to_string())
            .collect();
        let refs: Vec<&str> = strings.iter().map(String::as_str).collect();

        // Recompute EVERYTHING live from the row's own mk1 strings — never
        // trust the row's own `canonical_bytecode_hex`/`derived_csid` as
        // ground truth; they are the values under test.
        let card = decode(&refs).unwrap_or_else(|e| panic!("[{name}] decode failed: {e}"));
        let bytecode = encode_bytecode(&card)
            .unwrap_or_else(|e| panic!("[{name}] encode_bytecode failed: {e}"));
        let recomputed_id = derive_chunk_set_id(&bytecode);
        let recomputed_derived_hex = format!("{recomputed_id:05x}");

        let pinned_derived = row["derived_csid"]
            .as_str()
            .unwrap_or_else(|| panic!("[{name}] row.derived_csid is string"));
        assert_eq!(
            recomputed_derived_hex, pinned_derived,
            "[{name}] derived_csid does not reproduce \
             derive_chunk_set_id(encode_bytecode(decode(strings)))"
        );

        let pinned_bytecode_hex = row["canonical_bytecode_hex"]
            .as_str()
            .unwrap_or_else(|| panic!("[{name}] row.canonical_bytecode_hex is string"));
        assert_eq!(
            hex::encode(&bytecode),
            pinned_bytecode_hex,
            "[{name}] canonical_bytecode_hex does not reproduce encode_bytecode(decode(strings))"
        );

        let declared = row["declared_csid"]
            .as_str()
            .unwrap_or_else(|| panic!("[{name}] row.declared_csid is string"));
        let expect_mismatch = row["expect_mismatch_warning"]
            .as_bool()
            .unwrap_or_else(|| panic!("[{name}] row.expect_mismatch_warning is bool"));
        assert_eq!(
            expect_mismatch,
            declared != pinned_derived,
            "[{name}] expect_mismatch_warning disagrees with (declared_csid != derived_csid)"
        );

        // 5-hex-digit, zero-padded, lowercase per the spec's `{:05x}`
        // rendering rule (the token the sibling md-cli's `@i=` accepts).
        for (field, val) in [("declared_csid", declared), ("derived_csid", pinned_derived)] {
            assert_eq!(
                val.len(),
                5,
                "[{name}] {field} must render as exactly 5 hex digits, got {val:?}"
            );
            assert!(
                val.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "[{name}] {field} must be lowercase hex, got {val:?}"
            );
        }
    }
}

#[test]
fn standard_paths_table_fully_covered() {
    let doc = doc();
    let rows = rows(&doc);
    let sp_count = rows
        .iter()
        .filter(|r| {
            r["name"]
                .as_str()
                .map(|n| n.starts_with("SP"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(
        sp_count,
        STANDARD_PATHS.len(),
        "csid_ext corpus's SP## rows must cover every mk_codec::bytecode::STANDARD_PATHS \
         entry 1:1 — a table entry was added/removed without regenerating this corpus \
         (spec r4 L1-I1: this must trip a TEST, not a silent field warning)"
    );
}
