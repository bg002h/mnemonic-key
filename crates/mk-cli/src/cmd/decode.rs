//! `mk decode` — reassemble + decode one or more mk1 strings into a `KeyCard`.
//!
//! Realizes SPEC §3.5.2.

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

    /// Emit a structured JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk decode`.
pub fn run(args: DecodeArgs) -> Result<()> {
    let strings = read_mk1_strings(&args.mk1_strings)?;
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
    Ok(())
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
