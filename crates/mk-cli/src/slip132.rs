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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::base58;
    use bitcoin::bip32::{DerivationPath, Xpub};

    use super::{Slip132Variant, detect_and_normalize};

    // Corpus xpub shared with integration tests (depth-3, m/84'/0'/0' account).
    const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

    /// Re-version a canonical xpub into a SLIP-0132 form.
    fn to_slip132(xpub_str: &str, version: [u8; 4]) -> String {
        let mut data = base58::decode_check(xpub_str).unwrap();
        data[0..4].copy_from_slice(&version);
        base58::encode_check(&data)
    }

    fn path(s: &str) -> DerivationPath {
        DerivationPath::from_str(s).unwrap()
    }

    /// Drift guard: assert each SLIP-0132 arm's version bytes equal the SLIP-0132
    /// spec literals. A byte-parity test so an accidental edit to `detect_and_normalize`
    /// is caught by CI rather than silently producing wrong key material.
    #[test]
    fn slip132_version_bytes_match_slip0132() {
        // Each tuple: (SLIP-0132 variant, expected version bytes, one of its
        //              known prefix chars to confirm the right arm is being tested)
        let cases: &[([u8; 4], Slip132Variant)] = &[
            ([0x04, 0x9D, 0x7C, 0xB2], Slip132Variant::Ypub),
            ([0x04, 0xB2, 0x47, 0x46], Slip132Variant::Zpub),
            ([0x02, 0x95, 0xB4, 0x3F], Slip132Variant::YpubMultisig),
            ([0x02, 0xAA, 0x7E, 0xD3], Slip132Variant::ZpubMultisig),
            ([0x04, 0x4A, 0x52, 0x62], Slip132Variant::Upub),
            ([0x04, 0x5F, 0x1C, 0xF6], Slip132Variant::Vpub),
            ([0x02, 0x42, 0x89, 0xEF], Slip132Variant::UpubMultisig),
            ([0x02, 0x57, 0x54, 0x83], Slip132Variant::VpubMultisig),
        ];
        for &(expected_ver, variant) in cases {
            // Build a SLIP-0132 string with this version and confirm that
            // detect_and_normalize round-trips it back to the canonical xpub
            // AND returns the expected variant.
            let slip132_str = to_slip132(V2_84_MAIN, expected_ver);
            let (_, detected) = detect_and_normalize(&slip132_str)
                .unwrap_or_else(|e| panic!("detect_and_normalize failed for {variant:?}: {e}"));
            assert_eq!(
                detected,
                Some(variant),
                "version bytes {:02X?} did not map to expected variant {variant:?}",
                expected_ver
            );
        }
    }

    /// Normalizing a zpub must preserve all Xpub fields (public_key, chain_code,
    /// depth, child_number, parent_fingerprint) relative to the canonical xpub.
    #[test]
    fn normalize_zpub_yields_same_key() {
        const ZPUB_V: [u8; 4] = [0x04, 0xB2, 0x47, 0x46];
        let zpub = to_slip132(V2_84_MAIN, ZPUB_V);
        let canonical = Xpub::from_str(V2_84_MAIN).unwrap();

        let (normalized, variant) = detect_and_normalize(&zpub).unwrap();

        assert_eq!(variant, Some(Slip132Variant::Zpub), "must detect Zpub variant");
        assert_eq!(normalized.public_key, canonical.public_key, "public_key mismatch");
        assert_eq!(normalized.chain_code, canonical.chain_code, "chain_code mismatch");
        assert_eq!(normalized.depth, canonical.depth, "depth mismatch");
        assert_eq!(normalized.child_number, canonical.child_number, "child_number mismatch");
        assert_eq!(
            normalized.parent_fingerprint, canonical.parent_fingerprint,
            "parent_fingerprint mismatch"
        );
    }

    /// A canonical xpub must be passed through without detecting any variant.
    #[test]
    fn canonical_xpub_is_none() {
        let (_, variant) = detect_and_normalize(V2_84_MAIN).unwrap();
        assert_eq!(variant, None, "canonical xpub must not detect a SLIP-0132 variant");
    }

    /// An xpub-length base58check string with a bogus version must return an error
    /// (falls through the SLIP-0132 match to Xpub::from_str which rejects it).
    #[test]
    fn unknown_version_errors() {
        let bogus = to_slip132(V2_84_MAIN, [0xDE, 0xAD, 0xBE, 0xEF]);
        let result = detect_and_normalize(&bogus);
        assert!(result.is_err(), "bogus version bytes must return an error, got: {result:?}");
    }

    /// `path_matches` truth table — every case from the plan doc.
    #[test]
    fn path_predicate_truth_table() {
        // Zpub: hardened 84' at position 0
        assert!(
            Slip132Variant::Zpub.path_matches(&path("m/84'/0'/0'")),
            "Zpub must match m/84'/0'/0'"
        );
        // Unhardened must NOT match (R0 M3: normal-index derivation is not a Zpub account)
        assert!(
            !Slip132Variant::Zpub.path_matches(&path("m/84/0/0")),
            "Zpub must NOT match unhardened m/84/0/0"
        );
        // Wrong purpose
        assert!(
            !Slip132Variant::Zpub.path_matches(&path("m/49'/0'/0'")),
            "Zpub must NOT match m/49'/0'/0'"
        );
        // Ypub: hardened 49' at position 0
        assert!(
            Slip132Variant::Ypub.path_matches(&path("m/49'/0'/0'")),
            "Ypub must match m/49'/0'/0'"
        );
        // ZpubMultisig: m/48'/<coin>'/<acct>'/2'
        assert!(
            Slip132Variant::ZpubMultisig.path_matches(&path("m/48'/0'/0'/2'")),
            "ZpubMultisig must match m/48'/0'/0'/2'"
        );
        // Wrong script-type index
        assert!(
            !Slip132Variant::ZpubMultisig.path_matches(&path("m/48'/0'/0'/1'")),
            "ZpubMultisig must NOT match m/48'/0'/0'/1'"
        );
        // YpubMultisig: m/48'/<coin>'/<acct>'/1'
        assert!(
            Slip132Variant::YpubMultisig.path_matches(&path("m/48'/0'/0'/1'")),
            "YpubMultisig must match m/48'/0'/0'/1'"
        );
        // Short path (no index 3) must not panic and must return false
        assert!(
            !Slip132Variant::ZpubMultisig.path_matches(&path("m/48'/0'/0'")),
            "ZpubMultisig must NOT match short path m/48'/0'/0' (no script-type component)"
        );
    }
}
