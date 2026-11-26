//! `mk repair` — BCH error-correction for mk1 strings.
//!
//! Realizes plan §2.A.2 (v0.22.x follow-ups Tranche A.2'). Wraps
//! `mk_codec::string_layer::decode_string` (which already performs full
//! BCH correction up to t=4 per code variant) and renders a per-input
//! repair report.
//!
//! Single-HRP context (always `mk`): no `--hrp` flag and no Levenshtein
//! "did you mean" suggestion — `decode_string` validates HRP internally
//! against `mk_codec::consts::HRP`. An HRP mismatch is exit 2, as on every
//! other verb.
//!
//! Exit codes (constellation exit-code spec §3, decisions 6 and 7; D26's
//! meanings with the spec's numbers). The status is the sum of:
//!   - 16 (REPAIRED) — at least one correction was applied, and the
//!     corrected set reassembled (verified), or the set is incomplete and the
//!     rest of it verifies the correction when reassembled;
//!   - 4 (UNVERIFIED) — a corrected single-string card: nothing can verify it
//!     (so 20 with the 16);
//!   - 2 (INVALID) — a corrected or clean string that is not a chunk of the
//!     format (its header does not parse); it is still printed, with an error
//!     line on stderr.
//!
//! 0 means every input was already valid and decodes. Atomic failures exit
//! 2 with nothing on stdout: an uncorrectable string or any other codec error
//! out of the correcting decode (the bypass in [`run`]), and a complete chunk
//! set (a single-string card counts as one) that does not reassemble (`CliError::SetReassemblyMismatch` when a
//! correction was applied — THE FUNDS FIX: a per-chunk BCH correction
//! aliased to a DIFFERENT valid codeword — or `Invalid` when none was).
//!
//! Text output mirrors `mnemonic repair`'s text-form report shape (see
//! `mnemonic-toolkit/src/cmd/repair.rs::emit_repair_text`). JSON output
//! byte-matches the toolkit's standalone `RepairJson` schema (D27 — fields
//! `schema_version`, `kind`, `corrected_chunks`, `repairs`) so cross-CLI
//! parsers reuse the same struct.
//!
//! ## Set-level re-verify (Cycle E, F4 funds fix)
//!
//! codex32 BCH corrects <=4 substitutions; beyond that, bounded-distance
//! decoding can land a chunk within the radius-4 ball of a DIFFERENT valid
//! codeword (a miscorrection) — the per-chunk BCH check alone cannot tell
//! the difference. After the per-chunk correction loop below,
//! [`classify_mk1_set`] groups the corrected chunks by `chunk_set_id`
//! (parsed from each chunk's ALREADY-corrected string-layer header) and
//! reassembles every complete group through `mk_codec::decode`. See
//! `mnemonic-toolkit/design/SPEC_mk1_repair_set_level_reverify.md`.

use std::collections::{HashMap, HashSet};

use clap::Args;
use mk_codec::string_layer::StringLayerHeader;
use mk_codec::string_layer::bch::{ALPHABET, BchCode, DecodedString};
use serde::Serialize;

use crate::cmd::read_mk1_strings;
use crate::error::{CliError, Result};

/// `mk repair` arguments.
#[derive(Args, Debug)]
pub struct RepairArgs {
    /// One or more mk1 strings to attempt to repair. Use `-` to read
    /// one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
    /// the positional arguments. Display separators are stripped, so a card
    /// transcribed in grouped form re-ingests. SPEC §6b.
    #[arg(long = "in", value_name = "FILE")]
    pub in_file: Option<String>,

    /// Emit a single JSON envelope on stdout instead of the text-form
    /// report. Schema byte-matches `mnemonic repair --json`'s
    /// `RepairJson` shape (cross-CLI parser reuse).
    #[arg(long)]
    pub json: bool,
}

/// Per-input repair report. Mirrors toolkit's `RepairDetail` shape so
/// JSON output is byte-identical to `mnemonic repair --json`.
#[derive(Debug, Clone)]
struct RepairDetail {
    chunk_index: usize,
    original_chunk: String,
    corrected_chunk: String,
    /// `(position, was, now)` — `position` is 0-indexed into the data-part
    /// (chars after the `mk` HRP + `1` separator).
    corrected_positions: Vec<(usize, char, char)>,
}

/// Run `mk repair`.
pub fn run(args: RepairArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings, args.in_file.as_deref())?;
    let mut reports: Vec<RepairDetail> = Vec::with_capacity(strings.len());
    let mut corrected_chunks: Vec<String> = Vec::with_capacity(strings.len());
    // Cycle E: the post-correction string-layer header, parsed from the
    // SAME `decoded.data()` already computed below — no extra BCH decode
    // needed (`decoded.data()` is already the post-correction 5-bit symbol
    // stream). `Err` (a header-region parse failure) is recorded as a
    // `String` detail rather than hard-failing via `?`, mirroring the
    // toolkit's defensive `parse_mk1_group_key`: a chunk whose header does
    // not parse is no chunk of the format, corrected or not, and the run
    // reports it with INVALID while still printing the strings (exit-code
    // spec decision 6), rather than aborting the whole invocation.
    let mut header_results: Vec<std::result::Result<StringLayerHeader, String>> =
        Vec::with_capacity(strings.len());

    for (idx, original) in strings.iter().enumerate() {
        // An uncorrectable string (or any other codec error out of the
        // correcting decode: an HRP mismatch, a bad length) fails the whole
        // run atomically with 2 (spec R1). NO stdout output in this branch
        // except the `--json` error envelope: a repair that failed must not
        // emit a partial report. The message keeps its `mk: repair:` prefix.
        let decoded = match mk_codec::string_layer::decode_string(original) {
            Ok(d) => d,
            Err(e) => {
                let err = CliError::from(e);
                let code = err.exit_code();
                if args.json {
                    println!("{}", err.json_envelope(code));
                } else {
                    eprintln!("mk: repair: {}", err.message());
                }
                return Ok(code);
            }
        };
        let (corrected_chunk, corrected_positions) = reconstruct_corrected(original, &decoded);
        header_results.push(
            StringLayerHeader::from_5bit_symbols(decoded.data())
                .map(|(header, _consumed)| header)
                .map_err(|e| e.to_string()),
        );
        reports.push(RepairDetail {
            chunk_index: idx,
            original_chunk: original.clone(),
            corrected_chunk: corrected_chunk.clone(),
            corrected_positions,
        });
        corrected_chunks.push(corrected_chunk);
    }

    // Set-level re-verify and the per-string decode check (exit-code spec
    // decisions 6 and 7). A complete chunk set that does not reassemble
    // returns `Err` via `?` BEFORE any output is emitted (PM-r2-2 -- a batch
    // holding one suppresses ALL output, including any co-batched good
    // group); every other outcome prints, and its bits say what it was.
    let outcome = classify_mk1_set(&reports, &header_results, &corrected_chunks)?;

    if args.json {
        emit_json(&corrected_chunks, &reports)?;
    } else {
        emit_text(&corrected_chunks, &reports)?;
    }

    for line in &outcome.advisories {
        eprintln!("{line}");
    }

    // SPEC contract 2, `mk repair` coverage (r4 L2-I1/I2): the R2
    // stamped-vs-derived mismatch warning fires ONLY on the blessed
    // re-verify path above -- an already-valid supply and an incomplete
    // (partial/single-plate) supply never populate `blessed`, since
    // `classify_mk1_set` only pushes a CORRECTED group that reassembled.
    // No second decode (r4 L2-N1): `card` came straight out of that
    // re-verify's own `mk_codec::decode`.
    for (key, card) in &outcome.blessed {
        // A `SingleString` group is one whole (unchunked) card -- SPEC "The
        // comparison" has no declared id to compare there (single-string
        // encodings carry no chunk_set_id field), so it is silent by
        // construction.
        let GroupKey::Chunked(declared) = key else {
            continue;
        };
        if let Some(derived) = crate::cmd::derived_chunk_set_id(card) {
            if crate::cmd::chunk_set_id_warns(*declared, derived, card) {
                eprintln!(
                    "{} This id was set when the card was minted; the repair did not change it.",
                    crate::cmd::chunk_set_id_mismatch_warning(*declared, derived)
                );
            }
        }
    }

    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(outcome.bits.to_exit_code())
}

// ============================================================================
// Cycle E (`mk1-repair-set-level-reverify`) set-level re-verify, extended by
// the exit-code spec (decisions 6, 7). Mirrors `mnemonic-toolkit/src/repair.rs`'s
// `verify_mk1_set` (no shared crate -- separate binaries -- but the SAME
// `mk_codec` library, so grouping/decode semantics never drift).
// ============================================================================

/// What `mk repair` reports once every string passed the BCH layer.
struct RepairOutcome {
    /// The exit bits: REPAIRED for any correction used, UNVERIFIED for a
    /// corrected single-string card (no oracle), INVALID for any string that
    /// does not decode as a card or chunk.
    bits: crate::exit::Bits,
    /// Stderr lines printed after the report, in group order.
    advisories: Vec<String>,
    /// Every CORRECTED complete group that reassembled, with its card (the
    /// SPEC contract-2 comparison operand, plumbed out rather than
    /// re-decoded).
    blessed: Vec<(GroupKey, mk_codec::KeyCard)>,
}

/// Identifies one independent mk1 "card" within a `mk repair` batch
/// invocation. `Chunked` groups by the wire `chunk_set_id`; each
/// `SingleString`-headered chunk is its own singleton group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroupKey {
    Chunked(u32),
    SingleString(usize),
}

/// Human-readable identifier for a `GroupKey`, used in messages so a batch
/// invocation names WHICH group they are about.
fn describe_group_key(key: GroupKey) -> String {
    match key {
        GroupKey::Chunked(csid) => format!("chunk_set_id 0x{csid:05x}"),
        GroupKey::SingleString(idx) => format!("single-string chunk {idx}"),
    }
}

/// True iff `headers` (all members of ONE `GroupKey` group, in arbitrary
/// order) form a complete, internally-consistent chunk set: every index
/// `0..total_chunks` present EXACTLY once, with every member reporting the
/// SAME `total_chunks`. Discriminates on the PARSED indices, never on the
/// overloaded error string — a substitution that corrupts `total_chunks`
/// itself just misclassifies the group as incomplete, never as a false
/// confident success.
fn group_is_complete_and_consistent(headers: &[&StringLayerHeader]) -> bool {
    match headers.first() {
        Some(StringLayerHeader::SingleString { .. }) => headers.len() == 1,
        Some(StringLayerHeader::Chunked { total_chunks, .. }) => {
            let total = *total_chunks as usize;
            if headers.len() > total {
                return false; // more members than total_chunks declares
            }
            let mut seen = vec![false; total];
            for h in headers {
                match h {
                    StringLayerHeader::Chunked {
                        total_chunks: t,
                        chunk_index,
                        ..
                    } => {
                        if *t as usize != total {
                            return false; // inconsistent total_chunks within one csid
                        }
                        let idx = *chunk_index as usize;
                        if idx >= total || seen[idx] {
                            return false; // out-of-range or duplicate index
                        }
                        seen[idx] = true;
                    }
                    // Unreachable by construction (GroupKey partitions
                    // Chunked from SingleString) but handled defensively —
                    // fail-closed (NOT complete-and-consistent) rather than
                    // panicking on a future `#[non_exhaustive]` variant.
                    _ => return false,
                }
            }
            seen.iter().all(|&done| done)
        }
        // Empty group never occurs (built from >=1 member); a future
        // non-exhaustive header variant fails closed (not complete).
        _ => false,
    }
}

/// The set-level re-verify and per-string decode check over the FULL
/// supplied `corrected_chunks` (already per-string BCH-corrected), for every
/// group, corrected or not (decision 6: `repair` no longer exits 0 on a
/// string `mk decode` rejects).
///
/// - A chunk whose string-layer header does not parse is no chunk of the
///   format: INVALID, still printed.
/// - A **complete** chunk set is reassembled through `mk_codec::decode`.
///   Success with a correction is REPAIRED (verified). Failure is `Err`, and
///   nothing is printed: a corrected set that fails is
///   `SetReassemblyMismatch` (THE FUNDS FIX -- a per-chunk correction aliased
///   to a DIFFERENT valid codeword); a clean set that fails is `Invalid`.
/// - An **incomplete** chunk set with a correction is REPAIRED: the rest of
///   the set verifies it when it is reassembled (decision 7, D26).
/// - A **single-string** card is a complete set of one and is treated the
///   same way (a valid single-string mk1 cannot carry a real xpub, so in
///   practice it never decodes). If one ever does after a correction, it has
///   no cross-chunk check: REPAIRED + UNVERIFIED (20).
fn classify_mk1_set(
    reports: &[RepairDetail],
    header_results: &[std::result::Result<StringLayerHeader, String>],
    corrected_chunks: &[String],
) -> Result<RepairOutcome> {
    use crate::exit::{Bits, INVALID, REPAIRED, UNVERIFIED};

    let touched: HashSet<usize> = reports
        .iter()
        .filter(|r| !r.corrected_positions.is_empty())
        .map(|r| r.chunk_index)
        .collect();

    let mut bits = Bits::empty();
    let mut advisories: Vec<String> = Vec::new();
    let mut blessed: Vec<(GroupKey, mk_codec::KeyCard)> = Vec::new();
    let mut order: Vec<GroupKey> = Vec::new();
    let mut members: HashMap<GroupKey, Vec<usize>> = HashMap::new();

    for (i, res) in header_results.iter().enumerate() {
        match res {
            Ok(header) => {
                let key = match header {
                    StringLayerHeader::Chunked { chunk_set_id, .. } => {
                        GroupKey::Chunked(*chunk_set_id)
                    }
                    // `StringLayerHeader` is `#[non_exhaustive]`; a future
                    // third variant gets its own singleton group rather
                    // than silently mis-grouping.
                    _ => GroupKey::SingleString(i),
                };
                members
                    .entry(key)
                    .or_insert_with(|| {
                        order.push(key);
                        Vec::new()
                    })
                    .push(i);
            }
            Err(e) => {
                bits |= INVALID;
                if touched.contains(&i) {
                    bits |= REPAIRED;
                }
                advisories.push(format!(
                    "error: chunk {i} is not an mk1 chunk ({e}); the string \
                     printed for it is not a valid card"
                ));
            }
        }
    }

    for key in &order {
        let idxs = &members[key];
        let corrected = idxs.iter().any(|i| touched.contains(i));
        let refs: Vec<&str> = idxs.iter().map(|&i| corrected_chunks[i].as_str()).collect();
        let group = describe_group_key(*key);
        let headers: Vec<&StringLayerHeader> = idxs
            .iter()
            .map(|&i| {
                header_results[i]
                    .as_ref()
                    .expect("grouped only from Ok(..) parses")
            })
            .collect();
        if !group_is_complete_and_consistent(&headers) {
            if corrected {
                bits |= REPAIRED;
                advisories.push(format!(
                    "warning: corrected chunk(s) of an incomplete chunk set ({group}): the \
                     correction is verifiable when the rest of the set is reassembled — run \
                     `mk decode` with every chunk of the card to confirm it"
                ));
            }
            continue;
        }
        match mk_codec::decode(&refs) {
            Ok(card) if corrected => {
                bits |= REPAIRED;
                if matches!(key, GroupKey::SingleString(_)) {
                    // No cross-chunk hash: nothing verifies the correction.
                    bits |= UNVERIFIED;
                    advisories.push(format!(
                        "warning: correction UNVERIFIED ({group}) — a single-string card has \
                         no cross-chunk check, and a >4-error correction can alias to a \
                         different card; BIP-93 recommends confirming a corrected codex32 \
                         string"
                    ));
                }
                blessed.push((*key, card));
            }
            Ok(_) => {}
            Err(e) if corrected => {
                return Err(CliError::SetReassemblyMismatch {
                    group,
                    detail: e.to_string(),
                });
            }
            Err(e) => {
                return Err(CliError::Invalid(format!(
                    "{group}: the card is complete but does not decode as an mk1 card: {e}"
                )));
            }
        }
    }

    Ok(RepairOutcome {
        bits,
        advisories,
        blessed,
    })
}

/// Build the corrected mk1 string + `(position, was, now)` triples from
/// `DecodedString`. Uses `corrected_char_at` to get post-correction chars
/// (correctly handling the case where the correction lands inside the
/// BCH checksum region, mirroring toolkit's `repair_chunk_one` invariant).
fn reconstruct_corrected(
    original: &str,
    decoded: &DecodedString,
) -> (String, Vec<(usize, char, char)>) {
    let sep_pos = original
        .rfind('1')
        .expect("mk1 input passed BCH decode; must contain bech32 separator '1'");
    let (prefix, rest) = original.split_at(sep_pos);
    let data_part_raw: Vec<char> = rest[1..].chars().collect();

    let mut corrected_positions: Vec<(usize, char, char)> =
        Vec::with_capacity(decoded.corrected_positions.len());
    let mut data_chars = data_part_raw.clone();
    for &pos in &decoded.corrected_positions {
        let was = data_chars
            .get(pos)
            .copied()
            // Defensive: `decode_string` may report a corrected_position
            // inside the BCH checksum region (per `corrected_char_at` doc).
            // Render '?' as the "was" placeholder if the position is past
            // the data-part chars — checksum chars aren't in the input's
            // human-typed region anyway.
            .unwrap_or('?');
        let now = decoded.corrected_char_at(pos);
        if pos < data_chars.len() {
            data_chars[pos] = now;
        }
        corrected_positions.push((pos, was, now));
    }

    // Re-encode the corrected data + full checksum so output is a valid
    // mk1 string. Use `data_with_checksum` (already post-correction) to
    // ensure the emitted string is checksum-valid byte-exact with what
    // a re-encode of the underlying KeyCard would produce.
    //
    // M12: the `prefix` carries the input's original case (e.g. `MK` for the
    // canonical QR-friendly all-uppercase card), but the data symbols come
    // from the LOWERCASE `ALPHABET`. Splicing them verbatim would yield a
    // mixed-case `MK1<lowercase-data>` string that `decode_string` rejects
    // with `Error::MixedCase` — an un-ingestable repair artifact. mk-codec
    // emits lowercase canonically and accepts all-uppercase input, so we
    // normalize the prefix to lowercase here: the emitted string is then
    // uniformly all-lowercase (codec canonical) and round-trips cleanly.
    let mut out = String::with_capacity(prefix.len() + 1 + decoded.data_with_checksum.len());
    out.push_str(&prefix.to_lowercase());
    out.push('1');
    for &v in &decoded.data_with_checksum {
        out.push(ALPHABET[v as usize] as char);
    }

    // Suppress unused warning on `code` — surfaced in future enhancements
    // (per-chunk code-variant rendering in the text report); currently
    // implicit through the chunk-length.
    let _ = match decoded.code {
        BchCode::Regular => "regular",
        BchCode::Long => "long",
    };

    (out, corrected_positions)
}

/// Text-form report: `# Repair report` header, per-chunk correction lines,
/// then corrected chunks one per line. Mirrors toolkit's
/// `cmd::repair::emit_repair_text` shape byte-exact (modulo the `mk1`-only
/// `kind_str`).
fn emit_text(corrected_chunks: &[String], reports: &[RepairDetail]) -> Result<()> {
    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());
    if any_correction {
        println!("# Repair report");
        for r in reports {
            if r.corrected_positions.is_empty() {
                continue;
            }
            let n = r.corrected_positions.len();
            let plural = if n == 1 { "correction" } else { "corrections" };
            let mut line = format!("#   mk1 chunk {}: {} {} at ", r.chunk_index, n, plural);
            for (i, (pos, was, now)) in r.corrected_positions.iter().enumerate() {
                if i > 0 {
                    line.push_str(", ");
                }
                line.push_str(&format!("position {pos}: '{was}' -> '{now}'"));
            }
            println!("{line}");
        }
    }
    for chunk in corrected_chunks {
        println!("{chunk}");
    }
    Ok(())
}

// JSON envelope — schema MUST byte-match toolkit's standalone `RepairJson`
// at `mnemonic-toolkit/src/cmd/repair.rs:162-183` (D27 cross-CLI parser
// reuse). Field order is part of the schema (serde preserves struct field
// order in the default JSON serializer).
#[derive(Serialize)]
struct RepairJson<'a> {
    schema_version: &'static str,
    kind: &'static str,
    corrected_chunks: &'a [String],
    repairs: Vec<RepairJsonDetail<'a>>,
}

#[derive(Serialize)]
struct RepairJsonDetail<'a> {
    chunk_index: usize,
    original_chunk: &'a str,
    corrected_chunk: &'a str,
    corrected_positions: Vec<RepairJsonPosition>,
}

#[derive(Serialize)]
struct RepairJsonPosition {
    position: usize,
    was: String,
    now: String,
}

fn emit_json(corrected_chunks: &[String], reports: &[RepairDetail]) -> Result<()> {
    let envelope = RepairJson {
        schema_version: "1",
        kind: "mk1",
        corrected_chunks,
        repairs: reports
            .iter()
            // Mirror toolkit: only include entries for chunks that
            // actually had corrections applied.
            .filter(|r| !r.corrected_positions.is_empty())
            .map(|r| RepairJsonDetail {
                chunk_index: r.chunk_index,
                original_chunk: &r.original_chunk,
                corrected_chunk: &r.corrected_chunk,
                corrected_positions: r
                    .corrected_positions
                    .iter()
                    .map(|(p, w, n)| RepairJsonPosition {
                        position: *p,
                        was: w.to_string(),
                        now: n.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| CliError::Internal(format!("repair JSON serialize: {e}")))?;
    println!("{body}");
    Ok(())
}
