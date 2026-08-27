//! `mk decode` — reassemble + decode one or more mk1 strings into a `KeyCard`.
//!
//! Realizes the read side of SPEC §2 (String Layer) and §3 (Bytecode Layer).
//!
//! (Was a `SPEC §3.5.x` cite. The mk SPEC's §3.5 is "Origin path encoding"
//! and has no subsections -- that whole family of cites referred to nothing.
//! A 2026-06-10 audit repointed four of them; a 2026-08-21 review found a
//! fifth, and sweeping the class found nine. See FOLLOWUPS F-224.)

use clap::Args;
use mk_codec::KeyCard;
use serde_json::json;

use crate::cmd::{classify_code_variant, fmt_fingerprint, fmt_stub, read_mk1_strings};
use crate::error::{CliError, Result};

/// `mk decode` arguments.
#[derive(Args, Debug)]
pub struct DecodeArgs {
    /// One or more mk1 strings. Use `-` to read one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
    /// the positional arguments. Display separators are stripped, so a card
    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
    #[arg(long = "in", value_name = "FILE")]
    pub in_file: Option<String>,

    /// Emit a structured JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk decode`.
pub fn run(args: DecodeArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings, args.in_file.as_deref())?;
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs)?;
    let variant = strings
        .first()
        .map(|s| classify_code_variant(s))
        .unwrap_or("regular");

    if args.json {
        emit_json(&card, strings.len(), variant)?;
    } else {
        emit_text(&card, strings.len(), variant);
    }
    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(0)
}

fn emit_text(card: &KeyCard, chunks: usize, variant: &str) {
    println!("xpub:                {}", card.xpub);
    match &card.origin_fingerprint {
        Some(fp) => println!("origin_fingerprint:  {}", fmt_fingerprint(fp)),
        None => println!("origin_fingerprint:  (omitted, privacy-preserving mode)"),
    }
    println!("origin_path:         {}", card.origin_path);
    let stubs: Vec<String> = card.policy_id_stubs.iter().map(fmt_stub).collect();
    println!("policy_id_stubs:     {}", stubs.join(", "));
    println!("chunks:              {chunks} ({variant})");
}

fn emit_json(card: &KeyCard, chunks: usize, variant: &str) -> Result<()> {
    let stubs: Vec<String> = card.policy_id_stubs.iter().map(fmt_stub).collect();
    let envelope = json!({
        "schema_version": 1,
        "xpub": card.xpub.to_string(),
        "origin_fingerprint": card.origin_fingerprint.as_ref().map(fmt_fingerprint),
        "origin_path": card.origin_path.to_string(),
        "policy_id_stubs": stubs,
        "chunks": chunks,
        "code_variant": variant,
    });
    let s = serde_json::to_string(&envelope)
        .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
    println!("{s}");
    Ok(())
}
