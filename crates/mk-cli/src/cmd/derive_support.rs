//! Shared read-only public-derivation support for `mk address` + `mk derive`.
//!
//! Pure helpers — no private keys, no signing. `render_address` runs under a
//! `Secp256k1::verification_only()` context (no signing capability), structurally
//! reinforcing the no-private-key product boundary. Mirrors the toolkit's
//! `xpub_search/address_search.rs::render_address` (re-implemented locally; the
//! sibling rule forbids depending on the toolkit from a codec crate).

use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};
use bitcoin::secp256k1::{Secp256k1, VerifyOnly};
use bitcoin::{Address, KnownHrp, NetworkKind};

/// Single-key address script type. clap values: `p2pkh|p2sh-p2wpkh|p2wpkh|p2tr`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum AddressType {
    P2pkh,
    P2shP2wpkh,
    P2wpkh,
    P2tr,
}

impl AddressType {
    /// Stable kebab label for output (matches the clap value).
    pub fn label(self) -> &'static str {
        match self {
            AddressType::P2pkh => "p2pkh",
            AddressType::P2shP2wpkh => "p2sh-p2wpkh",
            AddressType::P2wpkh => "p2wpkh",
            AddressType::P2tr => "p2tr",
        }
    }
}

/// Network selector for address rendering. clap values: `mainnet|testnet|signet|regtest`.
/// An xpub's version bytes only distinguish `Main` vs `Test`; signet/regtest share the
/// test version bytes but differ in address HRP (`tb1…` vs `bcrt1…`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliNetwork {
    Mainnet,
    Testnet,
    Signet,
    Regtest,
}

impl CliNetwork {
    /// `NetworkKind` for legacy/p2sh address rendering (Main vs Test only).
    pub fn network_kind(self) -> NetworkKind {
        match self {
            CliNetwork::Mainnet => NetworkKind::Main,
            _ => NetworkKind::Test,
        }
    }

    /// Bech32 HRP for segwit/taproot rendering.
    pub fn known_hrp(self) -> KnownHrp {
        match self {
            CliNetwork::Mainnet => KnownHrp::Mainnet,
            CliNetwork::Testnet | CliNetwork::Signet => KnownHrp::Testnets,
            CliNetwork::Regtest => KnownHrp::Regtest,
        }
    }

    /// Stable lowercase label for output.
    pub fn label(self) -> &'static str {
        match self {
            CliNetwork::Mainnet => "mainnet",
            CliNetwork::Testnet => "testnet",
            CliNetwork::Signet => "signet",
            CliNetwork::Regtest => "regtest",
        }
    }
}

/// Outcome of address-type inference from a card's `origin_path`.
#[derive(Debug, PartialEq, Eq)]
pub enum AddressTypeInference {
    /// A recognized single-sig purpose at canonical account depth.
    Inferred(AddressType),
    /// A multisig cosigner path (BIP-48 `48'` / BIP-87 `87'`) — not addressable single-key.
    Multisig,
    /// Empty / non-hardened-purpose / non-standard / non-account-depth — cannot infer.
    Unknown,
}

/// Map a card's `origin_path` to an address-type inference.
///
/// Multisig (`48'`/`87'`) is detected at any depth. Single-sig purposes
/// (`44'`/`49'`/`84'`/`86'`) infer ONLY at canonical account depth
/// (`origin_path.len() == 3`) — keying off the purpose alone would emit garbage
/// addresses for a leaf card (e.g. `m/84'/0'/0'/0/5`), since `mk address` derives
/// `m/c/i` relative to whatever xpub the card holds.
pub fn infer_address_type(origin_path: &DerivationPath) -> AddressTypeInference {
    let comps: &[ChildNumber] = origin_path.as_ref();
    let purpose = match comps.first() {
        Some(ChildNumber::Hardened { index }) => *index,
        _ => return AddressTypeInference::Unknown, // empty or non-hardened purpose
    };
    if purpose == 48 || purpose == 87 {
        return AddressTypeInference::Multisig;
    }
    if comps.len() != 3 {
        return AddressTypeInference::Unknown; // account-depth gate
    }
    match purpose {
        44 => AddressTypeInference::Inferred(AddressType::P2pkh),
        49 => AddressTypeInference::Inferred(AddressType::P2shP2wpkh),
        84 => AddressTypeInference::Inferred(AddressType::P2wpkh),
        86 => AddressTypeInference::Inferred(AddressType::P2tr),
        _ => AddressTypeInference::Unknown,
    }
}

/// A verification-only secp context (no signing capability).
pub fn secp_verify() -> Secp256k1<VerifyOnly> {
    Secp256k1::verification_only()
}

/// Render a single address for a derived child xpub. Mirrors the toolkit's
/// `address_search.rs::render_address` builder set exactly.
pub fn render_address(
    secp: &Secp256k1<VerifyOnly>,
    child: &Xpub,
    ty: AddressType,
    net: CliNetwork,
) -> String {
    match ty {
        AddressType::P2pkh => Address::p2pkh(child.to_pub(), net.network_kind()).to_string(),
        AddressType::P2shP2wpkh => {
            Address::p2shwpkh(&child.to_pub(), net.network_kind()).to_string()
        }
        AddressType::P2wpkh => Address::p2wpkh(&child.to_pub(), net.known_hrp()).to_string(),
        AddressType::P2tr => {
            Address::p2tr(secp, child.to_x_only_pub(), None, net.known_hrp()).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // V2 bip84 mainnet account xpub (m/84'/0'/0') from the mk-codec v0.1 corpus.
    const ACCT_84_XPUB: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

    fn p(s: &str) -> DerivationPath {
        DerivationPath::from_str(s).unwrap()
    }

    #[test]
    fn address_type_kebab_values() {
        use clap::ValueEnum;
        assert_eq!(
            AddressType::P2shP2wpkh
                .to_possible_value()
                .unwrap()
                .get_name(),
            "p2sh-p2wpkh"
        );
        assert_eq!(
            AddressType::P2wpkh.to_possible_value().unwrap().get_name(),
            "p2wpkh"
        );
        assert_eq!(
            AddressType::P2pkh.to_possible_value().unwrap().get_name(),
            "p2pkh"
        );
        assert_eq!(
            AddressType::P2tr.to_possible_value().unwrap().get_name(),
            "p2tr"
        );
    }

    #[test]
    fn network_lower_values_and_hrp() {
        use clap::ValueEnum;
        assert_eq!(
            CliNetwork::Mainnet.to_possible_value().unwrap().get_name(),
            "mainnet"
        );
        assert_eq!(
            CliNetwork::Regtest.to_possible_value().unwrap().get_name(),
            "regtest"
        );
        assert_eq!(CliNetwork::Mainnet.known_hrp(), KnownHrp::Mainnet);
        assert_eq!(CliNetwork::Regtest.known_hrp(), KnownHrp::Regtest);
        assert_eq!(CliNetwork::Signet.known_hrp(), KnownHrp::Testnets);
        assert_eq!(CliNetwork::Mainnet.network_kind(), NetworkKind::Main);
        assert_eq!(CliNetwork::Testnet.network_kind(), NetworkKind::Test);
    }

    #[test]
    fn infer_account_depth_only() {
        use AddressTypeInference::*;
        assert_eq!(
            infer_address_type(&p("m/84'/0'/0'")),
            Inferred(AddressType::P2wpkh)
        );
        assert_eq!(
            infer_address_type(&p("m/44'/0'/0'")),
            Inferred(AddressType::P2pkh)
        );
        assert_eq!(
            infer_address_type(&p("m/49'/0'/0'")),
            Inferred(AddressType::P2shP2wpkh)
        );
        assert_eq!(
            infer_address_type(&p("m/86'/0'/0'")),
            Inferred(AddressType::P2tr)
        );
        // multisig at any depth
        assert_eq!(infer_address_type(&p("m/48'/0'/0'/2'")), Multisig);
        assert_eq!(infer_address_type(&p("m/87'/0'/0'")), Multisig);
        // leaf / over-deep → NOT inferred (account-depth gate)
        assert_eq!(infer_address_type(&p("m/84'/0'/0'/0/5")), Unknown);
        // empty / non-standard
        assert_eq!(infer_address_type(&p("m")), Unknown);
        assert_eq!(infer_address_type(&p("m/0/0")), Unknown);
        assert_eq!(infer_address_type(&p("m/9999'/1234'/56'/7'")), Unknown);
    }

    #[test]
    fn render_all_four_types() {
        let secp = secp_verify();
        let xpub = Xpub::from_str(ACCT_84_XPUB).unwrap();
        let child = xpub.derive_pub(&secp, &p("m/0/0")).unwrap();
        assert!(
            render_address(&secp, &child, AddressType::P2wpkh, CliNetwork::Mainnet)
                .starts_with("bc1q")
        );
        assert!(
            render_address(&secp, &child, AddressType::P2tr, CliNetwork::Mainnet)
                .starts_with("bc1p")
        );
        assert!(
            render_address(&secp, &child, AddressType::P2pkh, CliNetwork::Mainnet).starts_with('1')
        );
        assert!(
            render_address(&secp, &child, AddressType::P2shP2wpkh, CliNetwork::Mainnet)
                .starts_with('3')
        );
        assert!(
            render_address(&secp, &child, AddressType::P2wpkh, CliNetwork::Regtest)
                .starts_with("bcrt1q")
        );
        assert!(
            render_address(&secp, &child, AddressType::P2wpkh, CliNetwork::Testnet)
                .starts_with("tb1q")
        );
    }
}
