//! `mk verify` — BCH check + optional content match against expected fields.
//!
//! Realizes SPEC §3.5.4.

use clap::Args;
use mk_codec::KeyCard;
use serde_json::json;

use crate::cmd::{
    derive_stub_from_md1, fmt_fingerprint, fmt_stub, parse_derivation_path, parse_fingerprint,
    parse_stub_hex, parse_xpub, read_mk1_strings,
};
use crate::error::{CliError, Result};

/// `mk verify` arguments.
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// One or more mk1 strings. Use `-` to read one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Expected xpub. If supplied, must match the decoded card.
    #[arg(long)]
    pub xpub: Option<String>,

    /// Expected master fingerprint (8 hex chars).
    #[arg(long)]
    pub origin_fingerprint: Option<String>,

    /// Expected derivation path.
    #[arg(long)]
    pub origin_path: Option<String>,

    /// Expected `policy_id_stub` (repeatable; order-sensitive).
    #[arg(long)]
    pub policy_id_stub: Vec<String>,

    /// Expected `policy_id_stub` derived from md1 strings (repeatable; order-sensitive).
    #[arg(long)]
    pub from_md1: Vec<String>,

    /// Emit a JSON envelope on stdout instead of plain "OK"/error text.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk verify`.
pub fn run(args: VerifyArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings)?;
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs)?;

    if let Some(expected) = &args.xpub {
        let want = parse_xpub(expected)?;
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

    if let Some(expected) = &args.origin_path {
        let want = parse_derivation_path(expected)?;
        if want != card.origin_path {
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
    for md1 in &args.from_md1 {
        expected_stubs.push(derive_stub_from_md1(md1)?);
    }
    if !expected_stubs.is_empty() && expected_stubs != card.policy_id_stubs {
        let expected_fmt: Vec<String> = expected_stubs.iter().map(fmt_stub).collect();
        let actual_fmt: Vec<String> = card.policy_id_stubs.iter().map(fmt_stub).collect();
        return Err(CliError::ContentMismatch {
            field: "policy_id_stubs".into(),
            expected: expected_fmt.join(","),
            actual: actual_fmt.join(","),
        });
    }

    emit_ok(&card, &strings, args.json)?;
    Ok(0)
}

fn emit_ok(card: &KeyCard, strings: &[String], json_mode: bool) -> Result<()> {
    if json_mode {
        let envelope = json!({
            "schema_version": 1,
            "ok": true,
            "chunks": strings.len(),
            "policy_id_stubs": card.policy_id_stubs.iter().map(fmt_stub).collect::<Vec<_>>(),
        });
        let s = serde_json::to_string(&envelope)
            .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        println!(
            "OK: mk1 string(s) decode cleanly{}",
            expected_match_suffix(card)
        );
    }
    Ok(())
}

fn expected_match_suffix(_card: &KeyCard) -> &'static str {
    " (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)"
}
