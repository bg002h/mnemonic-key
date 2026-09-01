//! `mk verify` — BCH check + optional content match against expected fields.
//!
//! Realizes SPEC §3.3 (the form-aware `policy_id_stub`, which the `--from-md1`
//! comparison derives) and §5 (Linkage to MD, whose step 2 is the stub match
//! this performs).
//!
//! Was `§3.5.4`, which does not exist -- the document's deepest heading is
//! §3.7 and it has no `x.y.z` subsections. A 2026-06-10 audit repointed four
//! phantom `§3.5.1` cites to §3.3; this is a fifth of the same class, missed
//! then and found by an independent review 2026-08-21 (R6/F4).

use clap::Args;
use mk_codec::KeyCard;
use serde_json::json;

use crate::cmd::{
    chunk_set_id_comparison, derive_stub_from_md1_card, fmt_fingerprint, fmt_stub, group_md1_cards,
    parse_derivation_path, parse_fingerprint, parse_stub_hex, parse_xpub_normalized,
    read_mk1_strings, warn_chunk_set_id_mismatch,
};
use crate::error::{CliError, Result};

/// `mk verify` arguments.
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// One or more mk1 strings. Use `-` to read one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
    /// the positional arguments. Display separators are stripped, so a card
    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
    #[arg(long = "in", value_name = "FILE")]
    pub in_file: Option<String>,

    /// Expected xpub. If supplied, must match the decoded card.
    #[arg(long)]
    pub xpub: Option<String>,

    /// Expected master fingerprint (8 hex chars).
    #[arg(long)]
    pub origin_fingerprint: Option<String>,

    /// Expected derivation path.
    #[arg(long)]
    pub origin_path: Option<String>,

    /// Expected `policy_id_stub` (repeatable). Compared as a multiset, so the
    /// order given need not match the order the card carries.
    #[arg(long)]
    pub policy_id_stub: Vec<String>,

    /// Expected `policy_id_stub` derived from md1 strings (repeatable).
    /// Compared as a multiset; see `--policy-id-stub`.
    #[arg(long)]
    pub from_md1: Vec<String>,

    /// Emit a JSON envelope on stdout instead of plain "OK"/error text.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk verify`.
pub fn run(args: VerifyArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings, args.in_file.as_deref())?;
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs)?;
    // SPEC R2 (contract 2) + contract 4: recompute-and-warn on stderr like
    // the other four read verbs, AND report the pair on `emit_ok`'s own
    // stdout verdict / `--json` envelope below (contract 4 -- BOTH modes).
    let csid = chunk_set_id_comparison(&strings, &card);
    warn_chunk_set_id_mismatch(csid);

    // Parse origin_path once; both the xpub normalization check and the
    // content-match block below consume it (no double-parse).
    let want_path: Option<bitcoin::bip32::DerivationPath> = args
        .origin_path
        .as_deref()
        .map(parse_derivation_path)
        .transpose()?;

    if let Some(expected) = &args.xpub {
        let want = parse_xpub_normalized(expected, want_path.as_ref())?;
        if want != card.xpub {
            return Err(CliError::ContentMismatch {
                field: "xpub".into(),
                expected: want.to_string(),
                actual: card.xpub.to_string(),
            });
        }
    }

    if let Some(expected) = &args.origin_fingerprint {
        let want = parse_fingerprint(expected)?;
        match &card.origin_fingerprint {
            Some(got) if got == &want => {}
            Some(got) => {
                return Err(CliError::ContentMismatch {
                    field: "origin_fingerprint".into(),
                    expected: fmt_fingerprint(&want),
                    actual: fmt_fingerprint(got),
                });
            }
            None => {
                return Err(CliError::ContentMismatch {
                    field: "origin_fingerprint".into(),
                    expected: fmt_fingerprint(&want),
                    actual: "(omitted, privacy-preserving mode)".into(),
                });
            }
        }
    }

    if let Some(want) = &want_path {
        if *want != card.origin_path {
            return Err(CliError::ContentMismatch {
                field: "origin_path".into(),
                expected: want.to_string(),
                actual: card.origin_path.to_string(),
            });
        }
    }

    let mut expected_stubs: Vec<[u8; 4]> = Vec::new();
    for s in &args.policy_id_stub {
        expected_stubs.push(parse_stub_hex(s)?);
    }
    // One card per POLICY, not one per string: a keyed wallet policy always
    // arrives as a chunk set, so the values are grouped by chunk-set id first
    // and each GROUP contributes one stub.
    // Same normalization `encode` does: `md` prints display-grouped strings and
    // this CLI's own mk1 intake strips separators, so a grouped md1 must be
    // accepted here too. The fold applied this to `encode` ONLY, leaving the
    // two commands disagreeing about the same input (R7/B3).
    let from_md1: Vec<String> = args
        .from_md1
        .iter()
        .map(|s| crate::format::strip_display_separators(s))
        .collect();
    for card in group_md1_cards(&from_md1) {
        expected_stubs.push(derive_stub_from_md1_card(&card)?);
    }
    if !expected_stubs.is_empty() {
        // Compare as a MULTISET, not as an ordered list.
        //
        // Stub order on the wire is mint order, which is argument order -- but
        // the question `verify` answers is "is this card bound to these
        // policies", and that does not depend on the order they were typed.
        // The ordered comparison meant the SAME card checked against the SAME
        // policies in a different `--from-md1` order returned exit 4: a
        // CORRECT card reported as failing (R1, 2026-08-21). A false negative
        // here is expensive in a way a false positive is not -- it invites
        // re-engraving a good plate, or distrusting a sound backup.
        //
        // Sorting a copy rather than the originals keeps the ORDER available
        // for the note below, and multiset (not set) so a duplicated stub
        // still has to appear the same number of times.
        let mut want = expected_stubs.clone();
        let mut got = card.policy_id_stubs.clone();
        want.sort_unstable();
        got.sort_unstable();
        if want != got {
            let expected_fmt: Vec<String> = expected_stubs.iter().map(fmt_stub).collect();
            let actual_fmt: Vec<String> = card.policy_id_stubs.iter().map(fmt_stub).collect();
            return Err(CliError::ContentMismatch {
                field: "policy_id_stubs".into(),
                expected: expected_fmt.join(","),
                actual: actual_fmt.join(","),
            });
        }
        // Same stubs, different order: not a failure, but say so. Re-minting
        // with the chunk sets supplied in a different order produces a
        // different card on the wire, and an operator comparing two cards
        // byte-for-byte deserves to know why they differ.
        if expected_stubs != card.policy_id_stubs {
            eprintln!(
                "note: the card carries these stubs in a different order ({} vs {} as given); \
                 the binding is the same, but a re-mint in this order would be a different card",
                card.policy_id_stubs
                    .iter()
                    .map(fmt_stub)
                    .collect::<Vec<_>>()
                    .join(","),
                expected_stubs
                    .iter()
                    .map(fmt_stub)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
    }

    emit_ok(&card, &strings, args.json, csid)?;
    Ok(0)
}

fn emit_ok(
    card: &KeyCard,
    strings: &[String],
    json_mode: bool,
    csid: Option<(u32, u32)>,
) -> Result<()> {
    if json_mode {
        let mut envelope = json!({
            "schema_version": 1,
            "ok": true,
            "chunks": strings.len(),
            "policy_id_stubs": card.policy_id_stubs.iter().map(fmt_stub).collect::<Vec<_>>(),
        });
        // Contract 4: additive `chunk_set_id` object, present for chunked
        // input (declared vs. content-derived, always -- even on a match),
        // absent for single-string input (`csid == None`, nothing declared
        // to report). `schema_version` stays the integer 1; no other
        // envelope field changes.
        if let Some((declared, derived)) = csid {
            envelope["chunk_set_id"] = json!({
                "declared": format!("{declared:05x}"),
                "derived": format!("{derived:05x}"),
                "matches": declared == derived,
            });
        }
        let s = serde_json::to_string(&envelope)
            .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        println!(
            "OK: mk1 string(s) decode cleanly{}",
            expected_match_suffix(card)
        );
        // Contract 4: text mode's OK verdict must carry the mismatch on
        // STDOUT too (not just the stderr R2 warning above) -- same frozen
        // content (R6), so a consumer reading only stdout still sees it.
        if let Some((declared, derived)) = csid {
            if declared != derived {
                println!(
                    "{}",
                    crate::cmd::chunk_set_id_mismatch_warning(declared, derived)
                );
            }
        }
    }
    Ok(())
}

fn expected_match_suffix(_card: &KeyCard) -> &'static str {
    " (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)"
}
