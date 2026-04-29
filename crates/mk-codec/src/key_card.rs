//! `KeyCard` — the in-memory representation of a decoded MK card.
//!
//! Field semantics mirror the wire-format payload from
//! `design/SPEC_mk_v0_1.md` §3.2. The bytecode-layer encode/decode
//! lives in [`crate::bytecode`] (Phase 4); the string-layer wrapper
//! (BCH + chunking) wires up the public `encode`/`decode` functions
//! below in Phase 5.

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};

use crate::error::Result;

/// In-memory representation of one decoded MK card.
///
/// Per closure Q-8, `origin_fingerprint` is `Option<Fingerprint>`:
/// a card encoded with the bytecode-header fingerprint flag unset
/// (privacy-preserving mode) reconstructs to a `KeyCard` with
/// `origin_fingerprint = None`.
///
/// `#[non_exhaustive]` so future versions can add fields without
/// breaking external constructors.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCard {
    /// Policy ID stubs declaring which MD-encoded policy template(s)
    /// this xpub is intended to serve. Each stub is the top 4 bytes
    /// of the policy's `SHA-256(canonical_bytecode)`. The vector is
    /// guaranteed non-empty after a successful `decode` (the decoder
    /// rejects `count == 0` with `Error::InvalidPolicyIdStubCount`).
    pub policy_id_stubs: Vec<[u8; 4]>,

    /// Master-key fingerprint identifying the seed from which `xpub`
    /// was derived. Verbatim from BIP 380 origin notation `[fp/...]`.
    /// Optional per closure Q-8: encoders MAY omit (set bytecode-header
    /// bit 2 = 0) for the privacy-preserving mode.
    pub origin_fingerprint: Option<Fingerprint>,

    /// Derivation path from master to `xpub`. Encoded on the wire
    /// either via a 1-byte standard-path indicator (BIP 44/49/84/86/
    /// 48-segwit/48-nested/87 + testnet variants) or via the explicit
    /// `0xFE` escape hatch with LEB128 components.
    pub origin_path: DerivationPath,

    /// The BIP 32 extended public key. The wire format carries a
    /// 73-byte compact form (per closure Q-7); the in-memory `Xpub`
    /// is reconstructed at decode time using the locked rule:
    ///
    /// ```text
    /// depth        := component_count(origin_path)
    /// child_number := last_component(origin_path)
    /// ```
    pub xpub: Xpub,
}

/// Encode a `KeyCard` into one or more `mk1`-prefixed strings.
///
/// **Not yet implemented.** Calls Phase 5's string-layer pipeline once
/// it lands. Currently panics with `todo!()`.
pub fn encode(_card: &KeyCard) -> Result<Vec<String>> {
    todo!("mk-codec encode: string-layer pipeline lands in Phase 5; see design/IMPLEMENTATION_PLAN_mk_v0_1.md")
}

/// Decode one or more `mk1`-prefixed strings into a `KeyCard`.
///
/// **Not yet implemented.** Phase 5 wires this through BCH +
/// chunked-header reassembly + bytecode-layer decode.
pub fn decode(_strings: &[&str]) -> Result<KeyCard> {
    todo!("mk-codec decode: string-layer pipeline lands in Phase 5; see design/IMPLEMENTATION_PLAN_mk_v0_1.md")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity check: type signatures compile and the public API
    /// surface matches what the lib.rs re-exports expect. Real
    /// round-trip coverage at this layer lands in Phase 6.
    #[test]
    fn types_compile() {
        let _f: fn(&KeyCard) -> Result<Vec<String>> = encode;
        let _g: fn(&[&str]) -> Result<KeyCard> = decode;
    }
}
