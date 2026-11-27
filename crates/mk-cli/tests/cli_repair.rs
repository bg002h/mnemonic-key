//! Integration tests for `mk repair` (v0.4.0; Tranche A.2' of v0.22.x
//! follow-ups cycle per plan §4.A.2).
//!
//! Covers all 7 cells locked in the plan:
//!   1. `repair_already_valid_input_exits_0`
//!   2. `repair_one_substitution_exits_16`
//!   3. `repair_beyond_t4_capacity_exits_2`
//!   4. `repair_hrp_mismatch_exits_2` — NO Levenshtein suggestion (single-HRP)
//!   5. `repair_long_code_happy_path` — 108-data-part chunk corruption
//!   6. `repair_stdin_input_via_dash`
//!   7. `repair_json_envelope_shape` — schema byte-match with toolkit's
//!      `RepairJson` (cross-CLI parser reuse)
//!
//! Test fixtures generated inline via `mk_codec::encode` for self-contained
//! reproducibility (mirrors the `round_trip.rs` idiom).

use std::io::Write;
use std::process::{Command, Stdio};
use std::str::FromStr;

use assert_cmd::cargo::CommandCargoExt;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;
use mk_codec::string_layer::bch::ALPHABET;

/// Canonical V1 fixture from `crates/mk-codec/src/test_vectors/v0.1.json`
/// (same fixture as `round_trip.rs`).
const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V1_PATH: &str = "m/48'/0'/0'/2'";

/// Encode a canonical V1 KeyCard and return all chunks. Two-chunk emission
/// per the test fixture (chunk 0 = long code, chunk 1 = regular code).
fn generate_valid_mk1_chunks() -> Vec<String> {
    let xpub = Xpub::from_str(V1_XPUB).unwrap();
    let fp = Fingerprint::from([0xaa, 0xbb, 0xcc, 0xdd]);
    let path = DerivationPath::from_str(V1_PATH).unwrap();
    let stub = [0x11u8, 0x22, 0x33, 0x44];
    let card = KeyCard::new(vec![stub], Some(fp), path, xpub);
    mk_codec::encode(&card).expect("encode V1 KeyCard")
}

/// Flip the bech32 character at position `pos` (0-indexed into the data
/// part, i.e. chars after `mk1`). Returns the corrupted string. Replacement
/// is the next bech32-alphabet char (cyclically) — guarantees the result
/// is parseable but BCH-invalid. Mirrors toolkit's `repair.rs::flip_at`.
fn flip_at(chunk: &str, pos: usize) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let alphabet_str = std::str::from_utf8(ALPHABET).unwrap();
    let was_idx = alphabet_str.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = alphabet_str.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

fn flip_many(chunk: &str, positions: &[usize]) -> String {
    positions
        .iter()
        .fold(chunk.to_string(), |acc, &p| flip_at(&acc, p))
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 1: already-valid input → exit 0, no corrections.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_already_valid_input_exits_0() {
    let chunks = generate_valid_mk1_chunks();
    let valid = &chunks[0];
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["repair", valid])
        .output()
        .expect("invoke mk repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        0,
        "expected exit 0 for clean input; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(
        !stdout.contains("# Repair report"),
        "clean input must not emit a Repair report; got stdout={stdout:?}"
    );
    // The corrected chunk equals the input (pass-through).
    assert!(
        stdout.lines().any(|line| line == valid),
        "expected pass-through of valid input on stdout; got {stdout:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 2: one substitution → exit 16, 1 correction reported.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_one_substitution_exits_16() {
    let chunks = generate_valid_mk1_chunks();
    // Use the regular-code chunk (index 1) and flip 1 char at data-part pos 5.
    let valid = &chunks[1];
    let corrupted = flip_at(valid, 5);

    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["repair", &corrupted])
        .output()
        .expect("invoke mk repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        16,
        "expected exit 16 (REPAIRED); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(
        stdout.contains("# Repair report"),
        "expected `# Repair report` header; got {stdout:?}"
    );
    assert!(
        stdout.contains("mk1 chunk 0: 1 correction at position 5"),
        "expected per-chunk correction line at position 5; got {stdout:?}"
    );
    assert!(
        stdout.lines().any(|line| line == valid.as_str()),
        "expected corrected chunk to match the original valid mk1; got {stdout:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 3: 5+ substitutions exceed t=4 capacity → exit 2 (CliError::Codec).
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_beyond_t4_capacity_exits_2() {
    let chunks = generate_valid_mk1_chunks();
    // Spread positions so the BCH locator-degree exceeds 4; 5 flips.
    let valid = &chunks[1];
    let irreparable = flip_many(valid, &[3, 11, 19, 27, 35]);

    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["repair", &irreparable])
        .output()
        .expect("invoke mk repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        2,
        "expected exit 2 (CliError::Codec::BchUncorrectable); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    // mk-codec's BchUncorrectable Display is the surface here; the
    // exit code is the load-bearing assertion (D26).
    assert!(
        stderr.contains("BCH uncorrectable") || stderr.contains("uncorrectable"),
        "expected BCH-uncorrectable error message on stderr; got {stderr:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 4: HRP mismatch → exit 2 (CliError::Codec::InvalidHrp).
// NO Levenshtein suggestion since mk-cli has a single-HRP context (D26).
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_hrp_mismatch_exits_2() {
    // Take a valid mk1, swap its HRP to `ms` — keeps the data part
    // intact and parseable, but the HRP-bound polymod fires InvalidHrp.
    let chunks = generate_valid_mk1_chunks();
    let valid = &chunks[1];
    let hrp_swapped = valid.replacen("mk1", "ms1", 1);

    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["repair", &hrp_swapped])
        .output()
        .expect("invoke mk repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        2,
        "expected exit 2 (InvalidHrp); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid HRP") || stderr.contains("HRP"),
        "expected HRP error message on stderr; got {stderr:?}"
    );
    // D26 single-HRP context: no "did you mean" suggestion (the toolkit's
    // multi-HRP `mnemonic repair` adds Levenshtein suggestions; mk-cli MUST NOT).
    assert!(
        !stderr.contains("did you mean"),
        "single-HRP mk-cli must not emit Levenshtein suggestion; got {stderr:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 5: long-code (108-data-part) happy path — flip 1 char in the
// 108-data-part chunk and verify repair restores it.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_long_code_happy_path() {
    let chunks = generate_valid_mk1_chunks();
    // Chunk 0 has 108 data-part chars (long code).
    let valid_long = &chunks[0];
    assert_eq!(
        valid_long.len() - "mk1".len(),
        108,
        "fixture sanity: chunk 0 must be long-code (108 data-part chars); got len={}",
        valid_long.len()
    );

    // Flip 1 char inside the data region (avoid checksum tail — the data
    // region is the first 108-15 = 93 chars; pos 50 is comfortably inside).
    let corrupted = flip_at(valid_long, 50);

    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["repair", &corrupted])
        .output()
        .expect("invoke mk repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        16,
        "expected exit 16 for long-code 1-substitution repair; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(
        stdout.contains("mk1 chunk 0: 1 correction at position 50"),
        "expected long-code correction at position 50; got {stdout:?}"
    );
    assert!(
        stdout.lines().any(|line| line == valid_long.as_str()),
        "expected restored long-code chunk to match original; got {stdout:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 6: stdin input via `-` token — pipe one mk1 per line into
// `mk repair -` and verify the per-line repair fires.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_stdin_input_via_dash() {
    let chunks = generate_valid_mk1_chunks();
    // Two corrupted chunks, one per line on stdin.
    let bad_a = flip_at(&chunks[0], 50);
    let bad_b = flip_at(&chunks[1], 5);
    let stdin_body = format!("{bad_a}\n{bad_b}\n");

    let mut child = Command::cargo_bin("mk")
        .expect("mk binary")
        .args(["repair", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mk repair -");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_body.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait mk repair -");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        16,
        "expected exit 16 for stdin-with-corrupted-input; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    // Both chunks should be restored on stdout (one per line).
    assert!(
        stdout.lines().any(|line| line == chunks[0].as_str()),
        "expected restored chunk 0 on stdout; got {stdout:?}"
    );
    assert!(
        stdout.lines().any(|line| line == chunks[1].as_str()),
        "expected restored chunk 1 on stdout; got {stdout:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// M12: all-uppercase input (the canonical QR-friendly `MK1…` form) must
// re-emit a VALID, all-lowercase, re-ingestable mk1 — NOT a mixed-case
// `MK1<lowercase-data>` string that fails re-decode with `Error::MixedCase`.
//
// mk-codec rejects mixed case and accepts all-uppercase; the repair re-emit
// previously spliced the original-cased `MK` prefix with lowercase `ALPHABET`
// data, producing an un-ingestable artifact even on the clean (exit-0) path.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_uppercase_input_emits_redecodeable_lowercase() {
    let chunks = generate_valid_mk1_chunks();
    // Uppercase BOTH chunks to the canonical QR-friendly `MK1…` form. The V1
    // fixture emits a 2-chunk KeyCard, so a full re-decode needs both chunks.
    let upper: Vec<String> = chunks.iter().map(|c| c.to_uppercase()).collect();
    for u in &upper {
        assert!(
            u.starts_with("MK1"),
            "fixture sanity: uppercased mk1 starts with MK1; got {u:?}"
        );
    }

    // Repair the all-uppercase (clean) chunks via stdin (one per line) →
    // exit 0, no corrections.
    let stdin_body = format!("{}\n{}\n", upper[0], upper[1]);
    let mut child = Command::cargo_bin("mk")
        .expect("mk binary")
        .args(["repair", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mk repair -");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_body.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait mk repair -");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        0,
        "expected exit 0 for clean uppercase input; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    // The emitted corrected chunks (non-comment stdout lines).
    let emitted: Vec<String> = stdout
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        emitted.len(),
        2,
        "two inputs → two emitted chunks; got {emitted:?}"
    );

    // (a) Each emitted string MUST NOT be mixed-case. Either all-lowercase
    //     (codec canonical) or all-uppercase, but never `MK1<lowercase>`.
    for e in &emitted {
        let has_upper = e.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = e.chars().any(|c| c.is_ascii_lowercase());
        assert!(
            !(has_upper && has_lower),
            "emitted mk1 must not be mixed-case; got {e:?}"
        );
    }

    // (b) The emitted chunks MUST re-decode cleanly via `mk decode` — the
    //     load-bearing round-trip assertion (mixed-case would exit 2).
    let mut cmd2 = Command::cargo_bin("mk").expect("mk binary");
    let out2 = cmd2
        .args(["decode", "--json", &emitted[0], &emitted[1]])
        .output()
        .expect("invoke mk decode on repaired output");
    let code2 = out2.status.code().expect("exited normally");
    assert_eq!(
        code2,
        0,
        "repaired output must re-decode cleanly (no MixedCase); emitted={emitted:?} stderr={}",
        String::from_utf8_lossy(&out2.stderr)
    );

    // (c) It must decode to the SAME KeyCard as the original lowercase chunks.
    //     Compare the `xpub` field of both decodes.
    let xpub_orig = decode_json_xpub(&[&chunks[0], &chunks[1]]);
    let xpub_repaired = decode_json_xpub(&[&emitted[0], &emitted[1]]);
    assert_eq!(
        xpub_orig, xpub_repaired,
        "repaired uppercase input must decode to the same xpub as the original"
    );
}

/// Decode mk1 chunks via `mk decode --json` and return the `xpub` field.
fn decode_json_xpub(chunks: &[&String]) -> String {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    cmd.arg("decode").arg("--json");
    for c in chunks {
        cmd.arg(c.as_str());
    }
    let out = cmd.output().expect("invoke mk decode --json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "decode --json must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("decode --json output parses");
    v["xpub"].as_str().expect("xpub field").to_string()
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 7: JSON envelope shape — `--json <bad>` emits a `RepairJson`-shaped
// envelope (schema_version=1, kind=mk1, corrected_chunks, repairs).
// Schema byte-matches `mnemonic-toolkit/src/cmd/repair.rs::RepairJson`
// (D27 cross-CLI parser reuse).
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_json_envelope_shape() {
    let chunks = generate_valid_mk1_chunks();
    let valid = &chunks[1];
    let corrupted = flip_at(valid, 5);

    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["repair", "--json", &corrupted])
        .output()
        .expect("invoke mk repair --json");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        16,
        "expected exit 16 for JSON-mode repair; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout parses as JSON");

    // Schema mirror: byte-match with toolkit's `RepairJson` shape.
    assert_eq!(
        envelope["schema_version"],
        serde_json::Value::String("1".into()),
        "schema_version must equal \"1\" (string)"
    );
    assert_eq!(
        envelope["kind"],
        serde_json::Value::String("mk1".into()),
        "kind must equal \"mk1\""
    );

    let corrected_chunks = envelope["corrected_chunks"]
        .as_array()
        .expect("corrected_chunks must be a JSON array");
    assert_eq!(corrected_chunks.len(), 1, "one input → one corrected_chunk");
    assert_eq!(
        corrected_chunks[0],
        serde_json::Value::String(valid.clone()),
        "corrected_chunk must equal the original valid mk1"
    );

    let repairs = envelope["repairs"]
        .as_array()
        .expect("repairs must be a JSON array");
    assert_eq!(repairs.len(), 1, "one corrupted input → one repair entry");
    let r0 = &repairs[0];
    assert_eq!(r0["chunk_index"], serde_json::Value::from(0u32));
    assert_eq!(
        r0["original_chunk"],
        serde_json::Value::String(corrupted.clone())
    );
    assert_eq!(
        r0["corrected_chunk"],
        serde_json::Value::String(valid.clone())
    );

    let positions = r0["corrected_positions"]
        .as_array()
        .expect("corrected_positions must be a JSON array");
    assert_eq!(positions.len(), 1, "single-flip → one position entry");
    let p0 = &positions[0];
    assert_eq!(p0["position"], serde_json::Value::from(5u32));
    assert!(p0["was"].is_string(), "was must be a string");
    assert!(p0["now"].is_string(), "now must be a string");
    // The `was` character was at the corrupted-position; the `now` is the
    // restored char. They MUST differ.
    assert_ne!(p0["was"], p0["now"], "was != now for a real correction");
}

// ──────────────────────────────────────────────────────────────────────────
// Single-string group re-verify (baseline GAP-B, repair.rs
// `group_is_complete_and_consistent` SingleString arm).
//
// A valid single-string mk1 cannot carry a real xpub (65 bytes > the 56-byte
// single-string cap), so a corrected single string never decodes and 20
// (REPAIRED + UNVERIFIED) is unreachable in mk. A non-chunked string has no
// oracle: after a 1-char correction it is still printed, with a "do not
// trust it" error, at 16 + 4 + 2 = 22 (exit-code spec decision 6; the
// coordinator's ruling, matching md). A CHUNKED set of one follows the
// complete-set rule instead (exit_codes_spec.rs).
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_corrected_single_string_that_fails_decode_exits_22() {
    use mk_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};
    use mk_codec::string_layer::header::{StringLayerHeader, VERSION_V0_1};

    let header = StringLayerHeader::SingleString {
        version: VERSION_V0_1,
    };
    let mut data_5bit = header.to_5bit_symbols();
    data_5bit.extend(bytes_to_5bit(&[0xFF; 10])); // not a valid mk1 bytecode
    let valid = encode_5bit_to_string(&data_5bit).expect("encode single string");
    assert!(
        mk_codec::decode(&[valid.as_str()]).is_err(),
        "precondition: the BCH-valid single string must fail mk_codec::decode"
    );
    let corrupted = flip_at(&valid, 5);

    let out = Command::cargo_bin("mk")
        .expect("mk binary")
        .args(["repair", &corrupted])
        .output()
        .expect("invoke mk repair");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(22),
        "stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains(&valid),
        "the corrected string is still printed; got {stdout:?}"
    );
    assert!(
        stderr.contains("not chunked") && stderr.contains("Do not trust it"),
        "the error names a non-chunked string; got {stderr:?}"
    );
    assert!(
        !stderr.contains("reassemble"),
        "no set-reassembly wording for a non-chunked string; got {stderr:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Final review I-1: a group that is not a complete set and not a short set
// of distinct chunks (a duplicate, over-full, out-of-range or disagreeing
// totals) is MALFORMED: 2, nothing printed, the reason on stderr. Only a
// short set takes decision 7's "verifiable later" (16).
// ──────────────────────────────────────────────────────────────────────────

fn run_repair(args: &[&str]) -> std::process::Output {
    let mut a = vec!["repair"];
    a.extend_from_slice(args);
    Command::cargo_bin("mk")
        .expect("mk binary")
        .args(&a)
        .output()
        .expect("invoke mk repair")
}

fn assert_malformed(args: &[&str], reason: &str) {
    let out = run_repair(args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{args:?}: stderr={stderr}");
    assert!(
        out.stdout.is_empty(),
        "{args:?}: nothing printed; got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains(reason),
        "{args:?}: want {reason:?} in {stderr}"
    );
    assert!(
        !stderr.contains("verifiable when the rest of the set"),
        "{args:?}: no decision-7 advisory; got {stderr}"
    );
}

#[test]
fn repair_duplicate_chunk_a_a_is_malformed_2() {
    let c = generate_valid_mk1_chunks();
    assert_malformed(&[&c[0], &c[0]], "chunk index 0 appears twice");
}

#[test]
fn repair_over_full_a_b_b_is_malformed_2() {
    let c = generate_valid_mk1_chunks();
    assert_malformed(&[&c[0], &c[1], &c[1]], "3 chunks, but the set declares 2");
}

#[test]
fn repair_corrected_over_full_a_prime_b_b_is_malformed_2() {
    let c = generate_valid_mk1_chunks();
    let a = flip_at(&c[0], 10);
    assert_malformed(&[&a, &c[1], &c[1]], "3 chunks, but the set declares 2");
}

#[test]
fn repair_corrected_duplicate_a_prime_a_is_malformed_2() {
    let c = generate_valid_mk1_chunks();
    let a = flip_at(&c[0], 10);
    assert_malformed(&[&a, &c[0]], "chunk index 0 appears twice");
}

#[test]
fn repair_disagreeing_totals_is_malformed_2() {
    use mk_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};
    use mk_codec::string_layer::header::{StringLayerHeader, VERSION_V0_1};
    let chunk = |total: u8, index: u8| {
        let mut d = StringLayerHeader::Chunked {
            version: VERSION_V0_1,
            chunk_set_id: 0x0abcd,
            total_chunks: total,
            chunk_index: index,
        }
        .to_5bit_symbols();
        d.extend(bytes_to_5bit(&[0x11; 10]));
        encode_5bit_to_string(&d).unwrap()
    };
    let (a, b) = (chunk(2, 0), chunk(3, 1));
    assert_malformed(&[&a, &b], "disagree on the number of chunks");
}

/// The short set keeps decision 7: one corrected chunk of two is 16, with
/// the advisory, and `--json` marks it `"incomplete": true`.
#[test]
fn repair_short_set_is_16_and_json_marks_incomplete() {
    let c = generate_valid_mk1_chunks();
    let a = flip_at(&c[0], 10);
    let out = run_repair(&[&a]);
    assert_eq!(out.status.code(), Some(16));
    assert!(String::from_utf8_lossy(&out.stderr).contains("verifiable when the rest of the set"));
    let out = run_repair(&["--json", &a]);
    assert_eq!(out.status.code(), Some(16));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["incomplete"], true);
    assert!(v.get("not_a_card").is_none());
    // A complete set carries neither field.
    let out = run_repair(&["--json", &a, &c[1]]);
    assert_eq!(out.status.code(), Some(16));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v.get("incomplete").is_none() && v.get("not_a_card").is_none());
}

// ──────────────────────────────────────────────────────────────────────────
// Final review M-1: repair validates the card it decodes, as every other
// verb does. A card that fails `KeyCard::validate()` is still printed, at 2
// (clean) or 18 (corrected).
// ──────────────────────────────────────────────────────────────────────────
const LEGACY_I8: [&str; 2] = [
    "mk1qpeffzpqqqqaatd7allqqpp4sl8szqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqnehen8u5yagl2mw0p43h3j",
    "mk1qpeffzppl8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqte3jjj9mtsugd0x98xyvntv",
];

#[test]
fn repair_of_an_invalid_card_is_2_or_18_and_printed() {
    let out = run_repair(&LEGACY_I8);
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(LEGACY_I8[0]) && stdout.contains(LEGACY_I8[1]));
    assert!(String::from_utf8_lossy(&out.stderr).contains("this card is invalid"));
    let a = flip_at(LEGACY_I8[1], 10);
    let out = run_repair(&[LEGACY_I8[0], &a]);
    assert_eq!(out.status.code(), Some(18));
    assert!(String::from_utf8_lossy(&out.stdout).contains(LEGACY_I8[1]));
}

/// Final review M-3: `--json` at 22 names why the string is not a card.
#[test]
fn repair_json_not_a_card_at_22() {
    use mk_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};
    use mk_codec::string_layer::header::{StringLayerHeader, VERSION_V0_1};
    let mut d = StringLayerHeader::SingleString {
        version: VERSION_V0_1,
    }
    .to_5bit_symbols();
    d.extend(bytes_to_5bit(&[0xFF; 10]));
    let valid = encode_5bit_to_string(&d).unwrap();
    let out = run_repair(&["--json", &flip_at(&valid, 5)]);
    assert_eq!(out.status.code(), Some(22));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["not_a_card"]["kind"].is_string(), "{v}");
    assert!(v["not_a_card"]["message"].is_string(), "{v}");
    assert_eq!(v["corrected_chunks"][0], valid.as_str());
    // Field order is part of the schema: `not_a_card` right after `kind`,
    // where ms has it after `verdict` (mk has no `verdict`).
    let raw = String::from_utf8_lossy(&out.stdout);
    let at = |k: &str| raw.find(&format!("\"{k}\"")).unwrap();
    assert!(
        at("kind") < at("not_a_card") && at("not_a_card") < at("corrected_chunks"),
        "{raw}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Brian's ruling of 2026-11-26 (exit-code spec §2b item 3; FOLLOWUP
// `mk1-mixed-chunk-set-id-exits-0`): short groups under DIFFERENT
// `chunk_set_id`s that declare one total and together cover every index of
// it are one set split by a miscorrected id, or chunks of two cards mixed.
// Either way they are refused: 2, nothing printed. Only short groups count
// (a complete group is a card by itself), and short groups that do not cover
// their total keep decision 7 (0 clean, 16 corrected).
// ──────────────────────────────────────────────────────────────────────────

fn synthetic_chunk(chunk_set_id: u32, total: u8, index: u8) -> String {
    use mk_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};
    use mk_codec::string_layer::header::{StringLayerHeader, VERSION_V0_1};
    let mut d = StringLayerHeader::Chunked {
        version: VERSION_V0_1,
        chunk_set_id,
        total_chunks: total,
        chunk_index: index,
    }
    .to_5bit_symbols();
    d.extend(bytes_to_5bit(&[0x11; 10]));
    encode_5bit_to_string(&d).unwrap()
}

fn assert_split_set_refused(args: &[&str]) {
    let out = run_repair(args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{args:?}: stderr={stderr}");
    assert!(
        out.stdout.is_empty(),
        "{args:?}: nothing printed; got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains("together cover every index") && stderr.contains("miscorrect"),
        "{args:?}: the refusal names the split set; got {stderr}"
    );
    assert!(
        !stderr.contains("verifiable when the rest of the set"),
        "{args:?}: no decision-7 advisory; got {stderr}"
    );
}

#[test]
fn repair_two_set_ids_covering_every_index_is_2() {
    let (a, b) = (
        synthetic_chunk(0x0aaaa, 2, 0),
        synthetic_chunk(0x0bbbb, 2, 1),
    );
    assert_split_set_refused(&[&a, &b]);
    assert_split_set_refused(&[&b, &a]);
}

#[test]
fn repair_corrected_two_set_ids_covering_every_index_is_2() {
    let (a, b) = (
        synthetic_chunk(0x0aaaa, 2, 0),
        synthetic_chunk(0x0bbbb, 2, 1),
    );
    assert_split_set_refused(&[&flip_at(&a, 10), &b]);
    assert_split_set_refused(&[&a, &flip_at(&b, 10)]);
}

#[test]
fn repair_json_two_set_ids_covering_every_index_is_2_envelope_only() {
    let (a, b) = (
        synthetic_chunk(0x0aaaa, 2, 0),
        synthetic_chunk(0x0bbbb, 2, 1),
    );
    let out = run_repair(&["--json", &flip_at(&a, 10), &b]);
    assert_eq!(out.status.code(), Some(2));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v.get("corrected_chunks").is_none(),
        "an error envelope only, no repair report: {v}"
    );
}

#[test]
fn repair_three_set_ids_or_overlapping_groups_covering_every_index_is_2() {
    let (a, b, c) = (
        synthetic_chunk(0x0aaaa, 3, 0),
        synthetic_chunk(0x0bbbb, 3, 1),
        synthetic_chunk(0x0cccc, 3, 2),
    );
    assert_split_set_refused(&[&a, &b, &c]);
    // Overlap does not matter: {0, 1} under one id and {1, 2} under another.
    let (a1, b2) = (
        synthetic_chunk(0x0aaaa, 3, 1),
        synthetic_chunk(0x0bbbb, 3, 2),
    );
    assert_split_set_refused(&[&a, &a1, &b, &b2]);
}

#[test]
fn repair_short_groups_that_do_not_cover_keep_decision_7() {
    // Two ids, a set of 3, indices 0 and 1 only: two incomplete cards.
    let (a, b) = (
        synthetic_chunk(0x0aaaa, 3, 0),
        synthetic_chunk(0x0bbbb, 3, 1),
    );
    let out = run_repair(&[&a, &b]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run_repair(&[&flip_at(&a, 10), &b]);
    assert_eq!(out.status.code(), Some(16));
    // Different totals never cover one set: 0 of 2 and 1 of 3.
    let (c, d) = (
        synthetic_chunk(0x0aaaa, 2, 0),
        synthetic_chunk(0x0bbbb, 3, 1),
    );
    let out = run_repair(&[&c, &d]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
