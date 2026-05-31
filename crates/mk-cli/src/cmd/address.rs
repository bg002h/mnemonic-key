//! `mk address` — render N receive/change addresses controlled by a card's xpub.
//!
//! Read-only public derivation: decode the card, resolve the address type
//! (heuristic from the origin-path purpose at account depth, or explicit
//! `--address-type`; multisig cosigner cards are refused), derive `m/chain/index`
//! relative to the card's xpub, and render. No private keys, no signing.

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};
use clap::Args;
use mk_codec::KeyCard;
use serde_json::json;

use crate::cmd::derive_support::{
    AddressType, AddressTypeInference, CliNetwork, infer_address_type, render_address, secp_verify,
};
use crate::cmd::read_mk1_strings;
use crate::error::{CliError, Result};

/// `mk address` arguments.
#[derive(Args, Debug)]
pub struct AddressArgs {
    /// One or more mk1 strings. Use `-` to read one string per line from stdin.
    pub mk1_strings: Vec<String>,

    /// Address type. Defaults to the origin-path heuristic at account depth
    /// (m/44'→p2pkh, 49'→p2sh-p2wpkh, 84'→p2wpkh, 86'→p2tr); required otherwise.
    #[arg(long, value_enum)]
    pub address_type: Option<AddressType>,

    /// Number of addresses per chain, starting at index 0. Conflicts with `--range`.
    #[arg(long, conflicts_with = "range")]
    pub count: Option<u32>,

    /// Inclusive index range `A,B` (alternative to `--count`). Conflicts with `--count`.
    #[arg(long, conflicts_with = "count")]
    pub range: Option<String>,

    /// Which chain(s) to render. `receive` (0), `change` (1), or `both`.
    #[arg(long, value_enum, default_value = "receive")]
    pub chain: ChainSel,

    /// Network override. Defaults to the xpub's version bytes (mainnet/testnet);
    /// must agree with the xpub's network kind. Needed to distinguish signet/regtest.
    #[arg(long, value_enum)]
    pub network: Option<CliNetwork>,

    /// Emit a structured JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Chain selector for `mk address`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum ChainSel {
    Receive,
    Change,
    Both,
}

impl ChainSel {
    fn chains(self) -> &'static [u32] {
        match self {
            ChainSel::Receive => &[0],
            ChainSel::Change => &[1],
            ChainSel::Both => &[0, 1],
        }
    }
}

/// Run `mk address`.
pub fn run(args: AddressArgs) -> Result<u8> {
    let strings = read_mk1_strings(&args.mk1_strings)?;
    let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs)?;

    let addr_type = resolve_address_type(&card.origin_path, args.address_type)?;
    let network = resolve_network(&card.xpub, args.network)?;

    // I2 depth advisory: addresses are derived relative to the card's xpub; if it is
    // not at the canonical single-sig account depth (3), they may not match a wallet.
    if card.xpub.depth != 3 {
        eprintln!(
            "warning: card xpub is at depth {} (not the canonical account depth 3); \
             addresses are derived relative to this xpub and may not match a standard wallet",
            card.xpub.depth
        );
    }

    let indices = resolve_indices(args.count, args.range.as_deref())?;
    let secp = secp_verify();

    // (chain, index, address) in chain-major order (receive before change).
    let mut rows: Vec<(u32, u32, String)> = Vec::new();
    for &chain in args.chain.chains() {
        for &index in &indices {
            // `chain` is 0/1 (in-range); `index` is bounded < 2^31 by
            // `resolve_indices`, but map the error rather than `.unwrap()` (no
            // panic on user-derived values — C1).
            let dp: DerivationPath = vec![
                ChildNumber::from_normal_idx(chain).unwrap(),
                ChildNumber::from_normal_idx(index).map_err(|_| {
                    CliError::UsageError(format!(
                        "index {index} out of BIP-32 normal range (0..2147483647)"
                    ))
                })?,
            ]
            .into();
            let child = card.xpub.derive_pub(&secp, &dp).map_err(|e| {
                CliError::UsageError(format!("derivation failed at m/{chain}/{index}: {e}"))
            })?;
            rows.push((
                chain,
                index,
                render_address(&secp, &child, addr_type, network),
            ));
        }
    }

    if args.json {
        emit_json(&card, addr_type, network, &rows)?;
    } else {
        emit_text(args.chain, &rows);
    }
    Ok(0)
}

/// §3.1 address-type resolution: multisig-refuse FIRST, then explicit, then
/// account-depth heuristic, then require-explicit.
fn resolve_address_type(
    origin_path: &DerivationPath,
    explicit: Option<AddressType>,
) -> Result<AddressType> {
    match infer_address_type(origin_path) {
        AddressTypeInference::Multisig => Err(CliError::UsageError(format!(
            "card origin {origin_path} is a multisig cosigner xpub (BIP-48/BIP-87); single-key \
             addresses would not match the wallet. Use descriptor tooling for multisig address \
             derivation (e.g. `mnemonic verify-bundle`)"
        ))),
        inference => {
            if let Some(t) = explicit {
                return Ok(t);
            }
            match inference {
                AddressTypeInference::Inferred(t) => Ok(t),
                _ => Err(CliError::UsageError(format!(
                    "--address-type required: cannot infer an address type from card origin \
                     {origin_path} (only canonical single-sig account paths \
                     m/{{44,49,84,86}}'/<coin>'/<account>' auto-infer)"
                ))),
            }
        }
    }
}

/// Resolve the render network from the xpub version bytes + optional override.
/// The override must agree with the xpub's network KIND (main vs test).
fn resolve_network(xpub: &Xpub, explicit: Option<CliNetwork>) -> Result<CliNetwork> {
    let kind = xpub.network;
    match explicit {
        None => Ok(match kind {
            NetworkKind::Main => CliNetwork::Mainnet,
            NetworkKind::Test => CliNetwork::Testnet,
        }),
        Some(net) => {
            if net.network_kind() != kind {
                return Err(CliError::UsageError(format!(
                    "--network {} disagrees with the xpub's network ({}); refusing to render \
                     wrong-network addresses",
                    net.label(),
                    if kind == NetworkKind::Main {
                        "mainnet"
                    } else {
                        "testnet"
                    }
                )));
            }
            Ok(net)
        }
    }
}

/// `--count N` → `0..N`; `--range A,B` → `A..=B`; neither → `0..10`.
///
/// The highest generated index is validated against the BIP-32 normal-index
/// ceiling (`< 2^31`) before any allocation (C1): an out-of-range `--count` /
/// `--range` is a `UsageError` (exit 64), never a `from_normal_idx` panic or a
/// multi-gigabyte allocation.
fn resolve_indices(count: Option<u32>, range: Option<&str>) -> Result<Vec<u32>> {
    // Valid BIP-32 normal indices are 0..=2^31-1.
    const MAX_NORMAL: u32 = (1u32 << 31) - 1;
    match (count, range) {
        (Some(_), Some(_)) => unreachable!("clap conflicts_with"),
        (Some(c), None) => {
            // indices 0..c → highest is c-1; require c-1 <= MAX_NORMAL.
            if c > MAX_NORMAL.saturating_add(1) {
                return Err(CliError::UsageError(format!(
                    "--count {c} exceeds the BIP-32 normal-index ceiling (max 2147483648)"
                )));
            }
            Ok((0..c).collect())
        }
        (None, Some(r)) => {
            let (a, b) = r
                .split_once(',')
                .ok_or_else(|| CliError::UsageError(format!("--range expects `A,B`, got {r:?}")))?;
            let a: u32 = a
                .trim()
                .parse()
                .map_err(|e| CliError::UsageError(format!("--range start {a:?}: {e}")))?;
            let b: u32 = b
                .trim()
                .parse()
                .map_err(|e| CliError::UsageError(format!("--range end {b:?}: {e}")))?;
            if a > b {
                return Err(CliError::UsageError(format!(
                    "--range start {a} must be <= end {b}"
                )));
            }
            if b > MAX_NORMAL {
                return Err(CliError::UsageError(format!(
                    "--range end {b} exceeds the BIP-32 normal-index ceiling (2147483647)"
                )));
            }
            Ok((a..=b).collect())
        }
        (None, None) => Ok((0..10).collect()),
    }
}

fn emit_text(chain: ChainSel, rows: &[(u32, u32, String)]) {
    let grouped = matches!(chain, ChainSel::Both);
    let mut cur_chain: Option<u32> = None;
    for (c, idx, addr) in rows {
        if grouped && cur_chain != Some(*c) {
            let label = if *c == 0 { "receive" } else { "change" };
            println!("{label} (m/{c}/i):");
            cur_chain = Some(*c);
        }
        println!("  {idx}  {addr}");
    }
}

fn emit_json(
    card: &KeyCard,
    addr_type: AddressType,
    network: CliNetwork,
    rows: &[(u32, u32, String)],
) -> Result<()> {
    let addresses: Vec<_> = rows
        .iter()
        .map(|(c, i, a)| json!({ "chain": c, "index": i, "address": a }))
        .collect();
    let envelope = json!({
        "schema_version": 1,
        "xpub": card.xpub.to_string(),
        "origin_path": card.origin_path.to_string(),
        "address_type": addr_type.label(),
        "network": network.label(),
        "addresses": addresses,
    });
    let s = serde_json::to_string(&envelope)
        .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
    println!("{s}");
    Ok(())
}
