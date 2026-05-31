//! `mk derive` — derive a child xpub at a relative (unhardened) path from a card.
//!
//! Read-only public derivation: an xpub can only derive unhardened children
//! (it has no private key). The child xpub is composable — pipe it back into
//! `mk encode`. No signing.

use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChildNumber, DerivationPath};
use clap::Args;
use serde_json::json;

use crate::cmd::derive_support::secp_verify;
use crate::cmd::{fmt_fingerprint, read_mk1_strings};
use crate::error::{CliError, Result};

/// `mk derive` arguments. Exactly one of `--path` / `--index` is required.
#[derive(Args, Debug)]
#[command(group(
    clap::ArgGroup::new("target").required(true).args(["path", "index"])
))]
pub struct DeriveArgs {
    /// One or more mk1 strings. Use `-` to read one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Relative derivation path from the card's xpub, unhardened only (e.g. `m/0/5`).
    #[arg(long)]
    pub path: Option<String>,

    /// Single external-chain index: sugar for `--path m/0/<INDEX>`.
    #[arg(long)]
    pub index: Option<u32>,

    /// Emit a structured JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk derive`.
pub fn run(args: DeriveArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings)?;
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs)?;

    let rel: DerivationPath = match (args.path.as_deref(), args.index) {
        (Some(s), None) => parse_relative_unhardened(s)?,
        (None, Some(i)) => {
            // `--index N` = m/0/N. N must be a valid BIP-32 normal index (< 2^31);
            // map the error rather than `.unwrap()` so a huge index is exit 64, not a panic (C1).
            let leaf = ChildNumber::from_normal_idx(i).map_err(|_| {
                CliError::UsageError(format!(
                    "--index {i} out of BIP-32 normal range (0..2147483647)"
                ))
            })?;
            vec![ChildNumber::from_normal_idx(0).unwrap(), leaf].into()
        }
        // ArgGroup(required, !multiple) guarantees exactly one.
        _ => unreachable!("clap ArgGroup target is required + exclusive"),
    };

    let secp = secp_verify();
    let child = card
        .xpub
        .derive_pub(&secp, &rel)
        .map_err(|e| CliError::UsageError(format!("derivation failed at m/{rel}: {e}")))?;

    let network = match card.xpub.network {
        NetworkKind::Main => "mainnet",
        NetworkKind::Test => "testnet",
    };

    if args.json {
        let envelope = json!({
            "schema_version": 1,
            "parent_xpub": card.xpub.to_string(),
            "parent_origin_path": card.origin_path.to_string(),
            "relative_path": format!("m/{rel}"),
            "child_xpub": child.to_string(),
            "child_fingerprint": fmt_fingerprint(&child.fingerprint()),
            "depth": child.depth,
            "network": network,
        });
        let s = serde_json::to_string(&envelope)
            .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        println!("parent_xpub:          {}", card.xpub);
        println!("parent_origin_path:   {}", card.origin_path);
        println!("relative_path:        m/{rel}");
        println!("child_xpub:           {child}");
        println!(
            "child_fingerprint:    {}",
            fmt_fingerprint(&child.fingerprint())
        );
        println!("depth:                {}", child.depth);
        println!("network:              {network}");
    }
    Ok(0)
}

/// Parse a relative derivation path and reject any hardened component (an xpub
/// cannot derive hardened children — it has no private key).
fn parse_relative_unhardened(s: &str) -> Result<DerivationPath> {
    let path = DerivationPath::from_str(s)
        .map_err(|e| CliError::UsageError(format!("--path invalid derivation path {s:?}: {e}")))?;
    if path.as_ref().iter().any(ChildNumber::is_hardened) {
        return Err(CliError::UsageError(format!(
            "--path {s}: cannot derive hardened children from an xpub (no private key); \
             use only unhardened components"
        )));
    }
    Ok(path)
}
