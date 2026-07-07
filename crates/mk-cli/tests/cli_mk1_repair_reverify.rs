//! Cycle E (`mk1-repair-set-level-reverify`, F4), Phase P1 — `mk repair`
//! set-level re-verify.
//!
//! Mirrors the toolkit's P0 fix (`mnemonic-toolkit` commit `67723ae1`,
//! `crates/mnemonic-toolkit/tests/cli_mk1_repair_reverify.rs`) at the `mk
//! repair` CLI surface. Same `mk_codec` library (this repo IS the source of
//! `mk_codec`), so the pinned funds-anchor fixtures below alias IDENTICALLY.
//!
//! SPEC (`mnemonic-toolkit/design/SPEC_mk1_repair_set_level_reverify.md`) §2
//! tri-state, §3 exit-code semantics for `mk repair`:
//!   - Bless     (decode(group) == Ok)                    -> exit 5
//!   - Candidate (group incomplete — a single-plate repair) -> exit 5 + advisory
//!   - Reject    (group complete-and-consistent, decode == Err) -> exit 2 (FUNDS FIX)
//!
//! Multi-group batch folds to the dominant outcome (reject > candidate > bless).

use assert_cmd::Command;
use predicates::prelude::*;

/// Real ≥2-chunk mk1 card — chunk 0 (long code, 108-char data-part).
/// Pinned verbatim from the toolkit P0 fixture (same `mk_codec`, byte-for-byte
/// reusable — this repo IS the mk_codec source, so no re-derivation needed).
const CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
/// Same card — chunk 1 (regular code, 77-char data-part; the trailing
/// regular-code chunk the SPEC's funds anchor corrupts).
const CHUNK1: &str =
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

/// A SECOND, entirely distinct real mk1 card (different `chunk_set_id`).
/// Used ONLY as the "clean/other group" half of the batch-reject-dominant
/// test — never corrupted.
const SECOND_CHUNK0: &str = "mk1qpnd2wpqqsqek48ppe2rd4eyqvzg3vs7zfl2pe5jyqghcnaqxqq4gdatr9tn90ga6tg0purlfh9275f4pvjmck3usgpec7pzw3wvgsn9mwmd";
const SECOND_CHUNK1: &str =
    "mk1qpnd2wppha4qc2sv8g58zqcpswt0zfsza3lk237tx7xeg8evycaywffzk5r3hcma55t0u0d83tguz";

/// **Pinned funds-anchor seed (plan-R0 PI-2).** A 5-substitution corruption
/// of `CHUNK1`'s data-part that `bch_correct_regular` aliases to a
/// valid-but-DIFFERENT codeword, AND that fails full-set reassembly via
/// `mk_codec::decode(&[CHUNK0, <corrected>])`. COPIED VERBATIM from the
/// toolkit P0 fixture (`mnemonic-toolkit` commit `67723ae1`) — same
/// `mk_codec` crate at the same commit (`mnemonic-key main@85bca69`) means
/// this aliases IDENTICALLY; no re-derivation needed (no shared crate, but
/// the same library source).
///
/// If a future mk-codec BCH change invalidates this pinned alias, the
/// `full_set_miscorrection_*` tests below start failing at the
/// `mk decode`-equivalent assertion (exit != 2), not at a panic — re-pin by
/// re-running the toolkit's `find_mk1_miscorrection_seed` search tool
/// (`mnemonic-toolkit/tests/cli_mk1_repair_reverify.rs`) and copying the new
/// pinned string here too.
const CORRUPTED_MK1_CHUNK1: &str =
    "mk1qprsqhpp0f3kmtxzd65mvwcvr9usdatwxqvq6z70rgnwrgk6xndl8gy6nwa2n977sw6zh34rma0nh";

/// The pinned funds-anchor set: chunk 0 unmodified + the corrupted chunk 1.
const CORRUPTED_SET: [&str; 2] = [CHUNK0, CORRUPTED_MK1_CHUNK1];

// ──────────────────────────────────────────────────────────────────────────
// Bech32 char <-> 5-bit value helpers (self-contained; this file has no
// access to mk-cli's `pub(crate)` internals, only mk-codec's PUBLIC
// `ALPHABET`). Mirrors the `flip_at`/`flip_many` helpers in `cli_repair.rs`
// and the toolkit's `cli_mk1_repair_reverify.rs`.
// ──────────────────────────────────────────────────────────────────────────

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

/// Flip the bech32 char at `pos` (within the data-part) to the NEXT
/// alphabet char (cyclic) — a single deterministic substitution.
fn flip_at(chunk: &str, pos: usize) -> String {
    let (prefix, mut values) = data_part_values(chunk);
    let was = values[pos];
    values[pos] = (was + 1) % 32;
    rebuild_string(prefix, &values)
}

fn flip_many(chunk: &str, positions: &[usize]) -> String {
    positions
        .iter()
        .fold(chunk.to_string(), |acc, &p| flip_at(&acc, p))
}

// ============================================================================
// Sanity preconditions — the pinned fixtures are what they claim to be.
// ============================================================================

/// The pinned corrupted chunk1 differs from the original (else the "aliased
/// to a DIFFERENT codeword" claim is vacuous), and both CHUNK0/CHUNK1
/// genuinely reassemble as a real 2-chunk card.
#[test]
fn pinned_fixture_preconditions() {
    assert_ne!(
        CORRUPTED_MK1_CHUNK1, CHUNK1,
        "pinned corrupted chunk1 must differ from the original"
    );
    let card = mk_codec::decode(&[CHUNK0, CHUNK1]);
    assert!(
        card.is_ok(),
        "CHUNK0+CHUNK1 must be a genuine, reassembling 2-chunk mk1 card; got {card:?}"
    );
    // The corrupted set must NOT reassemble (the funds-relevant miscorrection).
    let corrupted_card = mk_codec::decode(&[CHUNK0, CORRUPTED_MK1_CHUNK1]);
    assert!(
        corrupted_card.is_err(),
        "the pinned CORRUPTED_MK1_CHUNK1 must alias to a codeword that fails full-set \
        reassembly with CHUNK0 — if this now succeeds, the pinned seed no longer aliases \
        under the current mk-codec BCH implementation; re-pin via the toolkit's \
        find_mk1_miscorrection_seed search tool"
    );
}

// ============================================================================
// Test 1 — manual-example per-plate PRESERVED (the C1 regression guard).
//
// `mk repair <single chunk of a 2-chunk card>` is the documented workflow
// (manual `44-mk-cli.md:247`; the shipped example there IS this same shape —
// a single long-code chunk of a real 2-chunk card). Must stay exit 5 with a
// loud "UNVERIFIED" advisory, NOT regress to exit 2.
// ============================================================================

#[test]
fn manual_example_single_plate_repair_stays_exit_5_with_advisory() {
    let bad_chunk1 = flip_at(CHUNK1, 25); // genuine 1-substitution, within t<=4
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", &bad_chunk1])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(CHUNK1))
        .stderr(predicate::str::contains("UNVERIFIED"))
        .stderr(predicate::str::contains("BIP-93"));
}

/// Same, via `--json` — the corrected chunk still appears in the JSON
/// envelope (exit 5), and the advisory still fires on stderr (PM-r2-4: a
/// `--json` Candidate is signalled by exit code + stderr advisory only, no
/// wire-shape change).
#[test]
fn manual_example_single_plate_repair_json_form_stays_exit_5_with_advisory() {
    let bad_chunk1 = flip_at(CHUNK1, 25);
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", "--json", &bad_chunk1])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(CHUNK1))
        .stderr(predicate::str::contains("UNVERIFIED"));
}

// ============================================================================
// Test 2 — full-set miscorrection REJECTED (THE FUNDS FIX).
// ============================================================================

#[test]
fn full_set_miscorrection_rejects_exit_2() {
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", CORRUPTED_SET[0], CORRUPTED_SET[1]])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(CORRUPTED_MK1_CHUNK1).not())
        .stderr(predicate::str::contains("does not reassemble"))
        .stderr(predicate::str::contains("chunk_set_id"));
}

/// `--json` form — the reject surfaces via the JSON error envelope on
/// stdout (mk-cli's `emit_error` routes JSON-mode errors to stdout, not
/// stderr), still exit 2, and does NOT emit a success envelope with the
/// miscorrected chunk.
#[test]
fn full_set_miscorrection_json_form_also_rejects_exit_2() {
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", "--json", CORRUPTED_SET[0], CORRUPTED_SET[1]])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("does not reassemble"))
        .stdout(predicate::str::contains("chunk_set_id"))
        .stdout(predicate::str::contains("\"error\""))
        .stdout(predicate::str::contains(CORRUPTED_MK1_CHUNK1).not());
}

// ============================================================================
// Test 3 — genuine <=4-error FULL-set correction still BLESSES (G1).
// ============================================================================

#[test]
fn genuine_t4_full_set_correction_still_blesses_exit_5() {
    let bad_chunk1 = flip_many(CHUNK1, &[3, 11, 19, 27]); // 4 spread errors
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", CHUNK0, &bad_chunk1])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(CHUNK0))
        .stdout(predicate::str::contains(CHUNK1))
        .stderr(predicate::str::contains("UNVERIFIED").not());
}

// ============================================================================
// Test 4 — BATCH reject-dominant (multi-group aggregation).
// ============================================================================

/// A single `mk repair` invocation spanning TWO `chunk_set_id` groups — {the
/// miscorrection group, a clean SECOND card's group} — must exit REJECT
/// (exit 2), and must NOT present EITHER group's chunks as recovered (a
/// batch success must never carry a miscorrection; PM-r2-2).
#[test]
fn batch_reject_dominates_over_clean_group() {
    Command::cargo_bin("mk")
        .unwrap()
        .args([
            "repair",
            CORRUPTED_SET[0],
            CORRUPTED_SET[1],
            SECOND_CHUNK0,
            SECOND_CHUNK1,
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("does not reassemble"))
        .stdout(predicate::str::contains(SECOND_CHUNK0).not())
        .stdout(predicate::str::contains(SECOND_CHUNK1).not());
}

/// Same batch shape, order-reversed (clean group first) — the dominant-Reject
/// fold must not depend on group order within the flat positional args.
#[test]
fn batch_reject_dominates_regardless_of_group_order() {
    Command::cargo_bin("mk")
        .unwrap()
        .args([
            "repair",
            SECOND_CHUNK0,
            SECOND_CHUNK1,
            CORRUPTED_SET[0],
            CORRUPTED_SET[1],
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(SECOND_CHUNK0).not())
        .stdout(predicate::str::contains(SECOND_CHUNK1).not());
}

// ============================================================================
// Test 5 — clean full set — exit 0, no advisory.
// ============================================================================

#[test]
fn clean_full_set_exit_0_no_report_no_advisory() {
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", CHUNK0, CHUNK1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stdout(predicate::str::contains(CHUNK0))
        .stdout(predicate::str::contains(CHUNK1))
        .stderr(predicate::str::contains("UNVERIFIED").not());
}

/// A clean but INCOMPLETE (single-chunk) supply must ALSO stay exit 0 — no
/// correction occurred, so there is no aliasing risk to flag (the tri-state
/// re-verify only ever engages a chunk_set_id group that had >= 1 chunk
/// actually corrected). Regression guard against over-eagerly
/// Candidate-flagging every incomplete supply regardless of correction.
#[test]
fn clean_partial_set_exit_0_no_advisory() {
    Command::cargo_bin("mk")
        .unwrap()
        .args(["repair", CHUNK0])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stderr(predicate::str::contains("UNVERIFIED").not());
}
