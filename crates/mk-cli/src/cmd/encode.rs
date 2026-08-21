//! `mk encode` — produce one or more mk1 strings from xpub + origin metadata.
//!
//! Realizes SPEC §3.3 (Policy ID stub format) from the v0.2 plan.

use clap::Args;
use mk_codec::KeyCard;
use mk_codec::string_layer::header::MAX_CHUNK_SET_ID;
use serde_json::json;

use crate::cmd::{
    classify_code_variant, derive_stub_from_md1_card, group_md1_cards, parse_derivation_path,
    parse_fingerprint, parse_stub_hex, parse_xpub_normalized,
};
use crate::error::{CliError, Result};

/// `mk encode` arguments.
#[derive(Args, Debug)]
pub struct EncodeArgs {
    /// BIP 32 extended public key (xpub-prefixed string).
    #[arg(long)]
    pub xpub: String,

    /// 8-hex-char master fingerprint. Mutually exclusive with `--privacy-preserving`.
    #[arg(long)]
    pub origin_fingerprint: Option<String>,

    /// Derivation path (e.g., `m/48'/0'/0'/2'`).
    #[arg(long)]
    pub origin_path: String,

    /// Repeatable. Each value is 8 lowercase hex chars (4 bytes).
    #[arg(long)]
    pub policy_id_stub: Vec<String>,

    /// Repeatable. Each value is an md1 string; the stub is derived per SPEC §3.3.
    #[arg(long)]
    pub from_md1: Vec<String>,

    /// Encode without master fingerprint. Mutually exclusive with `--origin-fingerprint`.
    #[arg(long)]
    pub privacy_preserving: bool,

    /// Force chunked output even when single-string would fit. (Reserved for v0.2;
    /// mk-codec auto-dispatches today.)
    #[arg(long)]
    pub force_chunked: bool,

    /// Force long-code BCH variant. (Reserved for v0.2; mk-codec auto-dispatches today.)
    #[arg(long)]
    pub force_long_code: bool,

    /// Pin the 20-bit `chunk_set_id` (hex, `0x` prefix optional) instead of
    /// deriving it from the payload. Chunked output only — single-string
    /// encodings carry no such field. For vector regeneration and conformance
    /// fixtures; the derived default is already deterministic, so ordinary
    /// encoding never needs this.
    #[arg(long)]
    pub chunk_set_id: Option<String>,

    /// Insert a separator every N characters in each emitted mk1 string
    /// (0 = unbroken). SPEC §3. Display only; --json stays unbroken.
    #[arg(long, default_value_t = 5)]
    pub group_size: u16,

    /// Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5.
    #[arg(long, default_value = "space", value_parser = crate::format::parse_separator)]
    pub separator: char,

    /// Emit a single JSON object on stdout instead of one mk1 string per line.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk encode`.
pub fn run(args: EncodeArgs) -> Result<u8> {
    if args.privacy_preserving && args.origin_fingerprint.is_some() {
        return Err(CliError::UsageError(
            "--privacy-preserving and --origin-fingerprint are mutually exclusive".into(),
        ));
    }

    let mut stubs: Vec<[u8; 4]> = Vec::new();
    for s in &args.policy_id_stub {
        stubs.push(parse_stub_hex(s)?);
    }
    // One card per POLICY, not one per string: a keyed wallet policy always
    // arrives as a chunk set, so the values are grouped by chunk-set id first
    // and each GROUP contributes one stub.
    for card in group_md1_cards(&args.from_md1) {
        stubs.push(derive_stub_from_md1_card(&card)?);
    }
    if stubs.is_empty() {
        return Err(CliError::UsageError(
            "at least one of --policy-id-stub or --from-md1 is required".into(),
        ));
    }

    let fingerprint = match (&args.origin_fingerprint, args.privacy_preserving) {
        (Some(s), false) => Some(parse_fingerprint(s)?),
        (None, true) => None,
        (None, false) => None,
        (Some(_), true) => unreachable!("guarded above"),
    };

    let path = parse_derivation_path(&args.origin_path)?;
    let xpub = parse_xpub_normalized(&args.xpub, Some(&path))?;

    let card = KeyCard::new(stubs, fingerprint, path, xpub);
    let strings = match &args.chunk_set_id {
        Some(s) => mk_codec::encode_with_chunk_set_id(&card, parse_chunk_set_id(s)?)?,
        None => mk_codec::encode(&card)?,
    };

    if args.json {
        emit_json(&strings)?;
    } else {
        for s in &strings {
            println!(
                "{}",
                crate::format::render_grouped(s, args.group_size as usize, args.separator)
            );
        }
    }
    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(0)
}

/// Parse `--chunk-set-id`: hex, `0x` prefix optional, and it must FIT.
///
/// A value over 20 bits is refused here rather than masked. Silently truncating
/// would emit a card under an id the caller did not ask for, and the caller's
/// whole reason for pinning is that the exact value matters.
fn parse_chunk_set_id(s: &str) -> Result<u32> {
    let body = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    let v = u32::from_str_radix(body, 16)
        .map_err(|e| CliError::UsageError(format!("--chunk-set-id: invalid hex {s:?}: {e}")))?;
    if v > MAX_CHUNK_SET_ID {
        return Err(CliError::UsageError(format!(
            "--chunk-set-id: {s:?} exceeds the 20-bit field (max 0x{MAX_CHUNK_SET_ID:05x})"
        )));
    }
    Ok(v)
}

fn emit_json(strings: &[String]) -> Result<()> {
    let variant = strings
        .first()
        .map(|s| classify_code_variant(s))
        .unwrap_or("regular");
    let envelope = json!({
        "schema_version": 1,
        "mk1_strings": strings,
        "chunk_count": strings.len(),
        "code_variant": variant,
    });
    let s = serde_json::to_string(&envelope)
        .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
    println!("{s}");
    Ok(())
}
