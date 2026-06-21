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
//!   - 5 — at least one input had corrections applied (REPAIR_APPLIED)
//!   - 2 — unrepairable input (`CliError::Codec(_)`) — propagated by `?`
//!
//! Text output mirrors `mnemonic repair`'s text-form report shape (see
//! `mnemonic-toolkit/src/cmd/repair.rs::emit_repair_text`). JSON output
//! byte-matches the toolkit's standalone `RepairJson` schema (D27 — fields
//! `schema_version`, `kind`, `corrected_chunks`, `repairs`) so cross-CLI
//! parsers reuse the same struct.

use clap::Args;
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
    let strings = read_mk1_strings(&args.mk1_strings)?;
    let mut reports: Vec<RepairDetail> = Vec::with_capacity(strings.len());
    let mut corrected_chunks: Vec<String> = Vec::with_capacity(strings.len());

    for (idx, original) in strings.iter().enumerate() {
        // `decode_string` performs BCH correction internally; HRP/length/
        // BCH-uncorrectable rejections surface as `mk_codec::Error` and
        // route to exit 2 via `CliError::Codec(_) => 2` in error.rs.
        let decoded = mk_codec::string_layer::decode_string(original)?;
        let (corrected_chunk, corrected_positions) = reconstruct_corrected(original, &decoded);
        reports.push(RepairDetail {
            chunk_index: idx,
            original_chunk: original.clone(),
            corrected_chunk: corrected_chunk.clone(),
            corrected_positions,
        });
        corrected_chunks.push(corrected_chunk);
    }

    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());

    if args.json {
        emit_json(&corrected_chunks, &reports)?;
    } else {
        emit_text(&corrected_chunks, &reports)?;
    }

    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(if any_correction { 5 } else { 0 })
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
