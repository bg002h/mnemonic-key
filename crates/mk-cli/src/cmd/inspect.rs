//! `mk inspect` — decode + structural commentary.
//!
//! Presents a card decoded per SPEC §2 + §3; the presentation itself is a CLI
//! surface with no SPEC section, pinned by its tests.
//!
//! v0.2 inspect output is intentionally less rich
//! than md-cli's: mk-codec's bytecode-layer surface isn't public yet
//! (see plan §3.7 — `bytecode` subcommand deferred to v0.3 alongside
//! the public-bytecode-API decision). Future enhancement opportunity.
//!
//! (Was a `SPEC §3.5.x` cite. The mk SPEC's §3.5 is "Origin path encoding"
//! and has no subsections -- that whole family of cites referred to nothing.
//! A 2026-06-10 audit repointed four of them; a 2026-08-21 review found a
//! fifth, and sweeping the class found nine. See FOLLOWUPS F-224.)

use bitcoin::bip32::ChildNumber;
use clap::Args;
use mk_codec::KeyCard;
use serde_json::json;

use crate::cmd::{classify_code_variant, fmt_fingerprint, fmt_stub, read_mk1_strings};
use crate::error::{CliError, Result};

/// `mk inspect` arguments.
#[derive(Args, Debug)]
pub struct InspectArgs {
    /// One or more mk1 strings. Use `-` to read one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
    /// the positional arguments. Display separators are stripped, so a card
    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
    #[arg(long = "in", value_name = "FILE")]
    pub in_file: Option<String>,

    /// Emit structured JSON on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk inspect`.
pub fn run(args: InspectArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings, args.in_file.as_deref())?;
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs)?;

    let per_chunk_variants: Vec<&str> = strings.iter().map(|s| classify_code_variant(s)).collect();

    if args.json {
        emit_json(&card, &strings, &per_chunk_variants)?;
    } else {
        emit_text(&card, &strings, &per_chunk_variants);
    }
    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(0)
}

fn xpub_fingerprint_hex(card: &KeyCard) -> String {
    fmt_fingerprint(&card.xpub.fingerprint())
}

fn path_components_text(card: &KeyCard) -> Vec<String> {
    card.origin_path
        .into_iter()
        .map(|c| match c {
            ChildNumber::Hardened { index } => format!("{index}h (hardened)"),
            ChildNumber::Normal { index } => format!("{index} (normal)"),
        })
        .collect()
}

fn emit_text(card: &KeyCard, strings: &[String], variants: &[&str]) {
    println!("xpub:                {}", card.xpub);
    match &card.origin_fingerprint {
        Some(fp) => println!("origin_fingerprint:  {}", fmt_fingerprint(fp)),
        None => println!("origin_fingerprint:  (omitted, privacy-preserving mode)"),
    }
    println!("xpub_fingerprint:    {}", xpub_fingerprint_hex(card));
    println!("origin_path:         {}", card.origin_path);
    for (i, comp) in path_components_text(card).into_iter().enumerate() {
        println!("  component[{i}]:       {comp}");
    }
    let stubs: Vec<String> = card.policy_id_stubs.iter().map(fmt_stub).collect();
    println!("policy_id_stubs:     {}", stubs.join(", "));
    println!("chunks:              {}", strings.len());
    for (i, v) in variants.iter().enumerate() {
        println!("  chunk[{i}]:           {v} (BCH variant)");
    }
}

fn emit_json(card: &KeyCard, strings: &[String], variants: &[&str]) -> Result<()> {
    let stubs: Vec<String> = card.policy_id_stubs.iter().map(fmt_stub).collect();
    let path_components = path_components_text(card);
    let envelope = json!({
        "schema_version": 1,
        "xpub": card.xpub.to_string(),
        "xpub_fingerprint": xpub_fingerprint_hex(card),
        "origin_fingerprint": card.origin_fingerprint.as_ref().map(fmt_fingerprint),
        "origin_path": card.origin_path.to_string(),
        "origin_path_components": path_components,
        "policy_id_stubs": stubs,
        "chunks": strings.len(),
        "chunk_variants": variants,
    });
    let s = serde_json::to_string(&envelope)
        .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
    println!("{s}");
    Ok(())
}
