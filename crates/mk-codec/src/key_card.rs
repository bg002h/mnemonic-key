//! `KeyCard` — the in-memory representation of a decoded MK card.
//!
//! Status: **type signatures only**. The `encode` and `decode` free
//! functions exist as placeholders so the public API surface is
//! discoverable; both currently `todo!()` and panic if called.
//! Implementation lands once the BCH and chunking primitives are
//! ported (forked from `md-codec` per `design/DECISIONS.md` D-13).

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};

use crate::error::Result;

/// In-memory representation of one decoded MK card.
///
/// Field layout mirrors the wire-format payload order from
/// `design/SPEC_mk_v0_1.md` §3.2, after the bytecode header and
/// stub count have been stripped:
///
/// 1. `policy_id_stubs` — one or more 4-byte Policy ID stubs.
/// 2. `origin_fingerprint` — the BIP 32 master fingerprint.
/// 3. `origin_path` — the derivation path from master to `xpub`.
/// 4. `xpub` — the BIP 32 extended public key.
///
/// Marked `#[non_exhaustive]` so future versions can add fields
/// (e.g., a "wallet name" hint, or a compact-xpub flag) without
/// breaking external constructors.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCard {
    /// Policy ID stubs declaring which MD-encoded policy template(s)
    /// this xpub is intended to serve. Each stub is the top 4 bytes of
    /// the policy's `SHA-256(canonical_bytecode)`. The vector is
    /// guaranteed non-empty after a successful `decode` (the decoder
    /// rejects `count == 0` with `Error::InvalidPolicyIdStubCount`).
    pub policy_id_stubs: Vec<[u8; 4]>,

    /// Master-key fingerprint identifying the seed from which `xpub`
    /// was derived. Verbatim from BIP 380 origin notation `[fp/...]`.
    pub origin_fingerprint: Fingerprint,

    /// Derivation path from master to `xpub`. Encoded on the wire
    /// either via a 1-byte standard-path indicator (BIP 44/49/84/86/
    /// 48-segwit/48-nested/87 + testnet variants) or via the explicit
    /// `0xFE` escape hatch with LEB128 components.
    pub origin_path: DerivationPath,

    /// The BIP 32 extended public key, in its full 78-byte form.
    /// `mk-codec` does not currently support compact xpub encoding
    /// (see `design/SPEC_mk_v0_1.md` §3.6 for rationale).
    pub xpub: Xpub,
}

/// Encode a `KeyCard` into one or more `mk1`-prefixed strings.
///
/// **Not yet implemented.** Calling this panics with `todo!()`.
///
/// When implemented, this function will produce a `Vec<String>` of
/// codex32-derived strings — one element if the encoded payload fits
/// in a single long-code string, multiple if chunking is required.
/// Per `design/SPEC_mk_v0_1.md` §2.4, multi-chunk MK is the norm.
pub fn encode(_card: &KeyCard) -> Result<Vec<String>> {
    todo!("mk-codec encode: implementation pending; see design/SPEC_mk_v0_1.md")
}

/// Decode one or more `mk1`-prefixed strings into a `KeyCard`.
///
/// **Not yet implemented.** Calling this panics with `todo!()`.
///
/// When implemented, this function will accept all chunks of a single
/// chunked MK card (in any order) or a single string for a
/// SingleString-typed MK card. Cross-chunk integrity verification
/// follows the chunking convention inherited from `md-codec`.
pub fn decode(_strings: &[&str]) -> Result<KeyCard> {
    todo!("mk-codec decode: implementation pending; see design/SPEC_mk_v0_1.md")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity check: `Error` and `KeyCard` types compile and the
    /// public function signatures match what the lib.rs re-exports
    /// expect. This test will be replaced with real round-trip
    /// coverage once `encode` and `decode` have implementations.
    #[test]
    fn types_compile() {
        // We can construct the type signature, just not call the
        // functions (they panic).
        let _f: fn(&KeyCard) -> Result<Vec<String>> = encode;
        let _g: fn(&[&str]) -> Result<KeyCard> = decode;
    }
}
