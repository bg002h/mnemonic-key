//! Integration tests for `mk repair` (v0.4.0; Tranche A.2' of v0.22.x
//! follow-ups cycle per plan §4.A.2).
//!
//! Covers all 7 cells locked in the plan:
//!   1. `repair_already_valid_input_exits_0`
//!   2. `repair_one_substitution_exits_5`
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
// Cell 2: one substitution → exit 5, 1 correction reported.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_one_substitution_exits_5() {
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
        5,
        "expected exit 5 (REPAIR_APPLIED); stderr={}",
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
        5,
        "expected exit 5 for long-code 1-substitution repair; stderr={}",
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
        5,
        "expected exit 5 for stdin-with-corrupted-input; stderr={}",
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
        5,
        "expected exit 5 for JSON-mode repair; stderr={}",
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
