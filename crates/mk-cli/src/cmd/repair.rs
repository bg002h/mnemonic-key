//! `mk repair` — BCH error-correction for mk1 strings.
//!
//! Realizes plan §2.A.2 (v0.22.x follow-ups Tranche A.2'). Wraps
//! `mk_codec::string_layer::decode_string` (which already performs full
//! BCH correction up to t=4 per code variant) and renders a per-input
//! repair report.
//!
//! Single-HRP context (always `mk`): no `--hrp` flag and no Levenshtein
//! "did you mean" suggestion — `decode_string` validates HRP internally
//! against `mk_codec::consts::HRP`. HRP mismatches surface as exit 2
//! (`CliError::Codec(mk_codec::Error::InvalidHrp(_))`).
//!
//! Exit codes (D26 cross-CLI parity):
//!   - 0 — every input was already valid (no corrections applied)
//!   - 5 — at least one input had corrections applied AND set-level
//!     re-verify either BLESSED the full corrected set (reassembles via
//!     `mk_codec::decode`) or the supplied set was a partial/single-plate
//!     group (documented per-plate workflow) — an UNVERIFIED advisory is
//!     printed to stderr in the latter case (Cycle E, `mk1-repair-set-level-
//!     reverify`, F4)
//!   - 2 — unrepairable input (`CliError::Codec(_)`), OR a corrected
//!     `chunk_set_id` group is complete-and-consistent but does NOT
//!     reassemble (`CliError::SetReassemblyMismatch` — THE FUNDS FIX: a
//!     per-chunk BCH correction aliased to a DIFFERENT valid codeword, not
//!     the original card) — propagated by `?`
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
//! the difference. After the per-chunk correction loop below, [`classify_mk1_set`]
//! groups the corrected chunks by `chunk_set_id` (parsed from each chunk's
//! ALREADY-corrected string-layer header — no extra decode needed, since
//! `decoded.data()` from the main loop is already post-correction) and, for
//! every group that had >=1 chunk actually corrected, re-verifies by
//! reassembling through `mk_codec::decode`:
//!   - **Bless** (`decode(group) == Ok`) — exit 5, unchanged from before.
//!   - **Candidate** (group incomplete — a single-plate/partial supply) —
//!     exit 5 + a loud advisory on stderr (the documented per-plate
//!     workflow, `44-mk-cli.md:247`, is PRESERVED).
//!   - **Reject** (group complete-and-consistent, `decode` returns `Err`) —
//!     exit 2, ALL output suppressed (a batch success must never carry a
//!     miscorrection).
//!
//! Multi-group batches fold to the dominant outcome (reject > candidate >
//! bless). See `mnemonic-toolkit/design/SPEC_mk1_repair_set_level_reverify.md`.

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
    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
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
    // toolkit's defensive `parse_mk1_group_key`: a parse failure on a
    // TOUCHED chunk is itself a Reject-worthy signal (SPEC §2 rule 2), but
    // an UNTOUCHED chunk failing to parse should never occur for genuine
    // encoder output and is simply skipped (ungrouped) rather than
    // aborting the whole invocation.
    let mut header_results: Vec<std::result::Result<StringLayerHeader, String>> =
        Vec::with_capacity(strings.len());

    for (idx, original) in strings.iter().enumerate() {
        // `decode_string` performs BCH correction internally; HRP/length/
        // BCH-uncorrectable rejections surface as `mk_codec::Error` and
        // route to exit 2 via `CliError::Codec(_) => 2` in error.rs.
        let decoded = mk_codec::string_layer::decode_string(original)?;
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

    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());

    // Cycle E set-level re-verify: only engage the classifier when at least
    // one chunk was actually corrected (an all-already-valid supply carries
    // no aliasing risk — mirrors the toolkit's `touched` gate). A dominant
    // Reject returns `Err` via `?` BEFORE any output is emitted (PM-r2-2 —
    // a batch that folds to Reject suppresses ALL output, including any
    // co-batched Bless/clean group).
    let mut unverified_advisory: Option<&'static str> = None;
    if any_correction {
        match classify_mk1_set(&reports, &header_results, &corrected_chunks)? {
            SetVerify::Blessed => {}
            SetVerify::Unverified => {
                unverified_advisory = Some(
                    "correction UNVERIFIED — reassemble the full card (`mk decode`) to confirm; \
                    a >4-error correction can alias to a different card; BIP-93 recommends \
                    confirming a corrected codex32 string",
                );
            }
        }
    }

    if args.json {
        emit_json(&corrected_chunks, &reports)?;
    } else {
        emit_text(&corrected_chunks, &reports)?;
    }

    if let Some(advisory) = unverified_advisory {
        eprintln!("warning: {advisory}");
    }

    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(if any_correction { 5 } else { 0 })
}

// ============================================================================
// Cycle E (`mk1-repair-set-level-reverify`) — SPEC §2 tri-state set-level
// re-verify classifier. Mirrors `mnemonic-toolkit/src/repair.rs`'s
// `verify_mk1_set` (no shared crate — separate binaries — but the SAME
// `mk_codec` library, so grouping/decode semantics never drift).
// ============================================================================

/// Per-`chunk_set_id`-group verdict (SPEC §2). Only ever computed for a
/// group that had at least one chunk actually corrected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupVerdict {
    /// `mk_codec::decode` on the exact supplied group returned `Ok`.
    Bless,
    /// The group is complete-and-consistent (every index `0..total_chunks`
    /// present exactly once, consistent `total_chunks`) but `decode`
    /// returned `Err` — the per-chunk correction(s) aliased to a DIFFERENT
    /// valid codeword. THE FUNDS FIX.
    Reject,
    /// The group is incomplete (a partial-set / single-plate repair) —
    /// cannot set-verify; preserved as an unverified candidate.
    Candidate,
}

/// The invocation-level outcome once at least one chunk was corrected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetVerify {
    /// Every touched group reassembled cleanly. No caveat.
    Blessed,
    /// At least one touched group was incomplete (Candidate) and none was a
    /// Reject — the caller still prints the correction but with a loud
    /// advisory (Reject dominates over Candidate, so this variant is only
    /// reached when no group Rejected).
    Unverified,
}

/// Identifies one independent mk1 "card" within a `mk repair` batch
/// invocation. `Chunked` groups by the wire `chunk_set_id`; each
/// `SingleString`-headered chunk is its own singleton group (unreachable
/// from real v0.1 encoders, per the SPEC's count=1 reachability note, but
/// handled uniformly rather than assumed away).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroupKey {
    Chunked(u32),
    SingleString(usize),
}

/// Human-readable identifier for a `GroupKey`, used in the `Reject` message
/// so a batch invocation names WHICH group failed.
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

/// Fold two `GroupVerdict`s to the dominant one across a multi-group batch
/// (SPEC §2 — `reject > candidate > bless`).
fn fold_verdict(acc: Option<GroupVerdict>, v: GroupVerdict) -> GroupVerdict {
    match (acc, v) {
        (Some(GroupVerdict::Reject), _) | (_, GroupVerdict::Reject) => GroupVerdict::Reject,
        (Some(GroupVerdict::Candidate), _) | (_, GroupVerdict::Candidate) => {
            GroupVerdict::Candidate
        }
        _ => GroupVerdict::Bless,
    }
}

/// SPEC §2 tri-state re-verify over the FULL supplied `corrected_chunks`
/// (already per-string BCH-corrected). Groups by `chunk_set_id`, classifies
/// only the groups that had at least one chunk actually corrected, then
/// folds to the dominant outcome.
///
/// Returns `Ok(SetVerify::Blessed)` / `Ok(SetVerify::Unverified)` on
/// Bless / Candidate; a dominant `Reject` returns
/// `Err(CliError::SetReassemblyMismatch)` — never `Ok` (a batch that folds
/// to Reject suppresses ALL output, including any co-batched Bless group).
fn classify_mk1_set(
    reports: &[RepairDetail],
    header_results: &[std::result::Result<StringLayerHeader, String>],
    corrected_chunks: &[String],
) -> Result<SetVerify> {
    let touched: HashSet<usize> = reports
        .iter()
        .filter(|r| !r.corrected_positions.is_empty())
        .map(|r| r.chunk_index)
        .collect();

    let mut order: Vec<GroupKey> = Vec::new();
    let mut members: HashMap<GroupKey, Vec<usize>> = HashMap::new();
    let mut dominant: Option<GroupVerdict> = None;
    let mut first_reject: Option<(String, String)> = None;

    for (i, res) in header_results.iter().enumerate() {
        match res {
            Ok(header) => {
                let key = match header {
                    StringLayerHeader::Chunked { chunk_set_id, .. } => {
                        GroupKey::Chunked(*chunk_set_id)
                    }
                    StringLayerHeader::SingleString { .. } => GroupKey::SingleString(i),
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
                if touched.contains(&i) {
                    if first_reject.is_none() {
                        first_reject =
                            Some((format!("chunk {i} (post-correction header)"), e.clone()));
                    }
                    dominant = Some(fold_verdict(dominant, GroupVerdict::Reject));
                }
                // An untouched chunk's header failing to parse should never
                // occur for genuine encoder output — skip it (ungrouped)
                // rather than reject on something that was never corrected.
            }
        }
    }

    for key in &order {
        let idxs = &members[key];
        if !idxs.iter().any(|i| touched.contains(i)) {
            // Untouched group — no aliasing risk, skip re-verify entirely.
            continue;
        }
        let headers: Vec<&StringLayerHeader> = idxs
            .iter()
            .map(|&i| {
                header_results[i]
                    .as_ref()
                    .expect("grouped only from Ok(..) parses")
            })
            .collect();
        let verdict = if !group_is_complete_and_consistent(&headers) {
            GroupVerdict::Candidate
        } else {
            let refs: Vec<&str> = idxs.iter().map(|&i| corrected_chunks[i].as_str()).collect();
            match mk_codec::decode(&refs) {
                Ok(_) => GroupVerdict::Bless,
                Err(e) => {
                    if first_reject.is_none() {
                        first_reject = Some((describe_group_key(*key), e.to_string()));
                    }
                    GroupVerdict::Reject
                }
            }
        };
        dominant = Some(fold_verdict(dominant, verdict));
    }

    match dominant.unwrap_or(GroupVerdict::Bless) {
        GroupVerdict::Bless => Ok(SetVerify::Blessed),
        GroupVerdict::Candidate => Ok(SetVerify::Unverified),
        GroupVerdict::Reject => {
            let (group, detail) =
                first_reject.expect("dominant Reject implies a recorded reject detail");
            Err(CliError::SetReassemblyMismatch { group, detail })
        }
    }
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
        .map_err(|e| CliError::UsageError(format!("repair JSON serialize: {e}")))?;
    println!("{body}");
    Ok(())
}
