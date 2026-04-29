//! Error type for `mk-codec`.
//!
//! Variants mirror the rejection conditions enumerated in
//! `design/mk/SPEC_mk_v0_1.md` §4 and `bip/bip-mnemonic-key.mediawiki`
//! "Decoder validity rules". All decoder-rejection paths in a future
//! implementation MUST surface one of these variants.

use thiserror::Error;

/// All errors `mk-codec` can produce.
///
/// Marked `#[non_exhaustive]` so that future versions can add variants
/// without breaking external callers' exhaustive `match` arms.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    /// HRP is not `mk` or input is not a valid bech32-shaped string.
    #[error("invalid HRP: {0}")]
    InvalidHrp(String),

    /// BCH checksum could not be corrected within the per-code-variant
    /// substitution capacity (4 for regular, 8 for long).
    #[error("BCH uncorrectable: {0}")]
    BchUncorrectable(String),

    /// Chunk-header card-type byte is not in {0x00 SingleString, 0x01 Chunked}.
    #[error("unsupported card type: {0}")]
    UnsupportedCardType(u8),

    /// Bytecode-header version != 0 in v0.1.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),

    /// A reserved bit in the bytecode header was set.
    #[error("reserved bits set in bytecode header")]
    ReservedBitsSet,

    /// `policy_id_stub_count == 0`. The spec requires ≥ 1.
    #[error("policy_id_stub_count must be >= 1")]
    InvalidPolicyIdStubCount,

    /// Origin-path indicator byte is outside the standard table or in
    /// the reserved range.
    #[error("invalid path indicator byte: {0:#04x}")]
    InvalidPathIndicator(u8),

    /// Explicit path declared `component_count > 32`.
    #[error("path too deep: {0} components (max 32)")]
    PathTooDeep(u8),

    /// A path component's encoded value is invalid (e.g., out of BIP 32
    /// range, or hardened-bit set in an invalid position).
    #[error("invalid path component: {0}")]
    InvalidPathComponent(String),

    /// xpub `version` field doesn't match a known network's xpub prefix.
    #[error("invalid xpub version: {0:#010x}")]
    InvalidXpubVersion(u32),

    /// xpub `depth` field is inconsistent with the encoded origin
    /// path's component count. Catches xpub-vs-path drift.
    #[error(
        "xpub depth ({xpub_depth}) does not match encoded path component count ({path_components})"
    )]
    XpubDepthMismatch {
        /// Depth field from the xpub serialization.
        xpub_depth: u8,
        /// Component count from the encoded origin path.
        path_components: u8,
    },

    /// Decoder hit end-of-stream mid-field.
    #[error("unexpected end of bytecode")]
    UnexpectedEnd,

    /// Decoder finished consuming all expected fields but bytes remain.
    #[error("trailing bytes after xpub")]
    TrailingBytes,
}

/// `Result` alias used throughout `mk-codec`.
pub type Result<T> = core::result::Result<T, Error>;
