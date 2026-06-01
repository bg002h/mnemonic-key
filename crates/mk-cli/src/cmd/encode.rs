//! `mk encode` — produce one or more mk1 strings from xpub + origin metadata.
//!
//! Realizes SPEC §3.5.1 from the v0.2 plan.

use clap::Args;
use mk_codec::KeyCard;
use serde_json::json;

use crate::cmd::{
    classify_code_variant, derive_stub_from_md1, parse_derivation_path, parse_fingerprint,
    parse_stub_hex, parse_xpub,
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

    /// Repeatable. Each value is an md1 string; the stub is derived per SPEC §3.5.1.
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
    for md1 in &args.from_md1 {
        stubs.push(derive_stub_from_md1(md1)?);
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
    let xpub = parse_xpub(&args.xpub)?;

    let card = KeyCard::new(stubs, fingerprint, path, xpub);
    let strings = mk_codec::encode(&card)?;

    if args.json {
        emit_json(&strings)?;
    } else {
        for s in &strings {
            println!("{s}");
        }
    }
    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(0)
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
