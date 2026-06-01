//! SLIP-0132 extended-key prefix acceptance (input normalization).
//!
//! Duplicates the CI-tested table from `mnemonic-toolkit/src/slip0132.rs`
//! (mk-cli is upstream of the toolkit and cannot depend on it; byte-parity is
//! guarded by `slip132_version_bytes_match_slip0132`). Decode-swap-reencode at
//! the base58check layer — key material is unchanged; only the 4 version bytes.

use std::str::FromStr;

use bitcoin::base58;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};

use crate::error::{CliError, Result};

const XPUB_MAINNET: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];
const TPUB_TESTNET: [u8; 4] = [0x04, 0x35, 0x87, 0xCF];

/// A detected non-canonical SLIP-0132 variant + its implied origin-path shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slip132Variant {
    /// `ypub` — mainnet BIP-49 P2SH-P2WPKH.
    Ypub,
    /// `zpub` — mainnet BIP-84 P2WPKH.
    Zpub,
    /// `Ypub` — mainnet BIP-48 P2WSH-P2SH multisig.
    YpubMultisig,
    /// `Zpub` — mainnet BIP-48 P2WSH multisig.
    ZpubMultisig,
    /// `upub` — testnet BIP-49 P2SH-P2WPKH.
    Upub,
    /// `vpub` — testnet BIP-84 P2WPKH.
    Vpub,
    /// `Upub` — testnet BIP-48 P2WSH-P2SH multisig.
    UpubMultisig,
    /// `Vpub` — testnet BIP-48 P2WSH multisig.
    VpubMultisig,
}

impl Slip132Variant {
    /// Display label for the stderr note, e.g. "zpub (BIP-84 P2WPKH)".
    pub fn label(self) -> &'static str {
        use Slip132Variant::*;
        match self {
            Ypub => "ypub (BIP-49 P2SH-P2WPKH)",
            Zpub => "zpub (BIP-84 P2WPKH)",
            YpubMultisig => "Ypub (BIP-48 P2WSH-P2SH multisig)",
            ZpubMultisig => "Zpub (BIP-48 P2WSH multisig)",
            Upub => "upub (testnet BIP-49 P2SH-P2WPKH)",
            Vpub => "vpub (testnet BIP-84 P2WPKH)",
            UpubMultisig => "Upub (testnet BIP-48 P2WSH-P2SH multisig)",
            VpubMultisig => "Vpub (testnet BIP-48 P2WSH multisig)",
        }
    }
    /// Canonical neutral form this variant normalizes to ("xpub" or "tpub").
    pub fn canonical_label(self) -> &'static str {
        use Slip132Variant::*;
        match self { Upub | Vpub | UpubMultisig | VpubMultisig => "tpub", _ => "xpub" }
    }
    /// Does `path` satisfy this variant's implied (HARDENED) shape?
    pub fn path_matches(self, path: &DerivationPath) -> bool {
        let c: &[ChildNumber] = path.as_ref();
        let h = |x: Option<&ChildNumber>, idx: u32|
            matches!(x, Some(ChildNumber::Hardened { index }) if *index == idx);
        use Slip132Variant::*;
        match self {
            Ypub | Upub => h(c.first(), 49),
            Zpub | Vpub => h(c.first(), 84),
            YpubMultisig | UpubMultisig => h(c.first(), 48) && h(c.get(3), 1),
            ZpubMultisig | VpubMultisig => h(c.first(), 48) && h(c.get(3), 2),
        }
    }
    /// Actionable remediation message when `path` does not match.
    pub fn mismatch_help(self, path: &DerivationPath) -> String {
        use Slip132Variant::*;
        let (expects, alt) = match self {
            Ypub | Upub => ("purpose 49' (e.g. m/49'/0'/0')", "supply the zpub/xpub for a different script type"),
            Zpub | Vpub => ("purpose 84' (e.g. m/84'/0'/0')", "supply the ypub for a 49' path"),
            YpubMultisig | UpubMultisig => ("m/48'/<coin>'/<account>'/1'", "use a Zpub for a 2' path, or xpub"),
            ZpubMultisig | VpubMultisig => ("m/48'/<coin>'/<account>'/2'", "use a Ypub for a 1' path, or xpub"),
        };
        format!(
            "SLIP-0132/origin-path mismatch — --xpub is a {} which expects --origin-path {}, but --origin-path is {}. \
             To engrave a backup, reconcile them: match the path to the prefix, or {}.",
            self.label(), expects, path, alt
        )
    }
}

/// Detect a SLIP-0132 prefix, normalize to canonical xpub/tpub, and parse.
/// Returns `(canonical Xpub, Some(variant))` for SLIP-0132 input,
/// `(Xpub, None)` for canonical xpub/tpub. Unrecognized versions fall through
/// to `Xpub::from_str`'s existing error.
pub fn detect_and_normalize(s: &str) -> Result<(Xpub, Option<Slip132Variant>)> {
    use Slip132Variant::*;
    let from_str = |s: &str| -> Result<Xpub> {
        Xpub::from_str(s).map_err(|e| CliError::UsageError(format!("invalid xpub {s:?}: {e}")))
    };
    let Ok(data) = base58::decode_check(s) else { return Ok((from_str(s)?, None)); };
    if data.len() < 4 { return Ok((from_str(s)?, None)); }
    let ver: [u8; 4] = data[0..4].try_into().unwrap();
    let (swap, variant) = match ver {
        [0x04, 0x9D, 0x7C, 0xB2] => (XPUB_MAINNET, Ypub),
        [0x04, 0xB2, 0x47, 0x46] => (XPUB_MAINNET, Zpub),
        [0x02, 0x95, 0xB4, 0x3F] => (XPUB_MAINNET, YpubMultisig),
        [0x02, 0xAA, 0x7E, 0xD3] => (XPUB_MAINNET, ZpubMultisig),
        [0x04, 0x4A, 0x52, 0x62] => (TPUB_TESTNET, Upub),
        [0x04, 0x5F, 0x1C, 0xF6] => (TPUB_TESTNET, Vpub),
        [0x02, 0x42, 0x89, 0xEF] => (TPUB_TESTNET, UpubMultisig),
        [0x02, 0x57, 0x54, 0x83] => (TPUB_TESTNET, VpubMultisig),
        _ => return Ok((from_str(s)?, None)),
    };
    let mut swapped = data;
    swapped[0..4].copy_from_slice(&swap);
    let reencoded = base58::encode_check(&swapped);
    Ok((from_str(&reencoded)?, Some(variant)))
}
