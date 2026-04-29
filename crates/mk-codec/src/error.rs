//! Error type for `mk-codec`.
//!
//! Variants mirror the rejection conditions enumerated in
//! `design/SPEC_mk_v0_1.md` §4 ("Bytecode-Validity Rules") and
//! `bip/bip-mnemonic-key.mediawiki` §"Decoder validity rules". All
//! decoder-rejection paths in a future implementation MUST surface
//! one of these variants. Pre-BIP-submission, every variant is
//! required to map to at least one named negative test vector
//! (tracked as `decoder-error-variant-parity` in
//! `design/FOLLOWUPS.md`).

use thiserror::Error;

/// All errors `mk-codec` can produce.
///
/// Marked `#[non_exhaustive]` so that future versions can add variants
/// without breaking external callers' exhaustive `match` arms.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    // ── String-layer errors (codex32 plumbing, HRP, chunk-header) ───────────

    /// HRP is not `mk` or input is not a valid bech32-shaped string.
    #[error("invalid HRP: {0}")]
    InvalidHrp(String),

    /// BCH checksum could not be corrected within the per-code-variant
    /// substitution capacity (4 for regular, 8 for long).
    #[error("BCH uncorrectable: {0}")]
    BchUncorrectable(String),

    /// Chunk-header card-type byte is not in {0x00 SingleString, 0x01 Chunked}.
    /// The 5-bit type field's reserved range 0x02..=0x1F MUST be rejected.
    #[error("unsupported card type: 0x{0:02x}")]
    UnsupportedCardType(u8),

    /// 5-bit payload symbols, after BCH verification, do not byte-align
    /// (i.e., the trailing pad bits of the final 5-bit symbol are non-zero).
    /// Parallels md1's `MalformedPayloadPadding` rejection.
    #[error("malformed payload padding (5-bit symbols don't byte-align)")]
    MalformedPayloadPadding,

    /// For chunked input: chunks have inconsistent `chunk_set_id` values.
    /// Used at reassembly time to detect mixed-card-set inputs.
    #[error("chunk_set_id mismatch across chunks")]
    ChunkSetIdMismatch,

    /// For chunked input: malformed chunked-string header (e.g., total_chunks
    /// = 0 or > 32, chunk_index >= total_chunks, gaps or duplicates in the
    /// index sequence at reassembly).
    #[error("chunked-header malformed: {0}")]
    ChunkedHeaderMalformed(String),

    /// For chunked input: reassembled bytecode's trailing 4-byte
    /// `cross_chunk_hash` does not match `SHA-256(canonical_bytecode)[0..4]`.
    #[error("cross-chunk integrity hash mismatch")]
    CrossChunkHashMismatch,

    // ── Bytecode-layer errors (after string-layer reassembly) ────────────────

    /// Bytecode-header version != 0 in v0.1.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),

    /// A reserved bit in the bytecode header was set (bits 0, 1, 3 in v0.1;
    /// bit 2 is the fingerprint flag and is allowed).
    #[error("reserved bits set in bytecode header")]
    ReservedBitsSet,

    /// Bytecode-header fingerprint flag (bit 2) and `origin_fingerprint`
    /// payload presence disagree: either bit 2 is set but
    /// `origin_fingerprint` is absent from the payload, or bit 2 is
    /// unset but `origin_fingerprint` was emitted.
    #[error("fingerprint flag does not match payload presence")]
    FingerprintFlagMismatch,

    /// `policy_id_stub_count == 0`. The spec requires ≥ 1.
    #[error("policy_id_stub_count must be >= 1")]
    InvalidPolicyIdStubCount,

    /// Origin-path indicator byte is outside the standard table or in the
    /// reserved range. (Per SPEC §3.5: 0x00, 0x08-0x10, 0x16, 0x18-0xFD,
    /// 0xFF are reserved; 0x16 is reserved pending md1 dictionary update,
    /// see FOLLOWUPS `md-path-dictionary-0x16-gap`.)
    #[error("invalid path indicator byte: 0x{0:02x}")]
    InvalidPathIndicator(u8),

    /// Explicit path declared `component_count > MAX_PATH_COMPONENTS`
    /// (closure Q-3 lock: max 10, was 32 in the pre-closure draft).
    #[error("path too deep: {0} components (max 10)")]
    PathTooDeep(u8),

    /// A path component's encoded value is invalid (e.g., out of BIP 32
    /// range, or hardened-bit set in an invalid position).
    #[error("invalid path component: {0}")]
    InvalidPathComponent(String),

    /// xpub `version` field doesn't match a known network's xpub prefix.
    #[error("invalid xpub version: 0x{0:08x}")]
    InvalidXpubVersion(u32),

    /// Decoder hit end-of-stream mid-field.
    #[error("unexpected end of bytecode")]
    UnexpectedEnd,

    /// Decoder finished consuming all expected fields but bytes remain.
    #[error("trailing bytes after xpub")]
    TrailingBytes,
}

/// `Result` alias used throughout `mk-codec`.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Each variant carries enough information for its rendered Display
    /// to be diagnostic. Sanity-check the format strings render
    /// non-empty for the parameterized variants.
    #[test]
    fn parameterized_variants_render() {
        let cases: Vec<(Error, &str)> = vec![
            (
                Error::InvalidHrp("ms".into()),
                "invalid HRP: ms",
            ),
            (
                Error::UnsupportedCardType(0x05),
                "unsupported card type: 0x05",
            ),
            (
                Error::UnsupportedVersion(1),
                "unsupported version: 1",
            ),
            (
                Error::InvalidPathIndicator(0x16),
                "invalid path indicator byte: 0x16",
            ),
            (
                Error::PathTooDeep(11),
                "path too deep: 11 components (max 10)",
            ),
            (
                Error::InvalidXpubVersion(0xDEADBEEF),
                "invalid xpub version: 0xdeadbeef",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(format!("{err}"), expected);
        }
    }

    /// Unparameterized variants render their static message verbatim.
    #[test]
    fn static_variants_render() {
        assert_eq!(
            format!("{}", Error::ReservedBitsSet),
            "reserved bits set in bytecode header",
        );
        assert_eq!(
            format!("{}", Error::FingerprintFlagMismatch),
            "fingerprint flag does not match payload presence",
        );
        assert_eq!(
            format!("{}", Error::CrossChunkHashMismatch),
            "cross-chunk integrity hash mismatch",
        );
        assert_eq!(
            format!("{}", Error::ChunkSetIdMismatch),
            "chunk_set_id mismatch across chunks",
        );
        assert_eq!(
            format!("{}", Error::MalformedPayloadPadding),
            "malformed payload padding (5-bit symbols don't byte-align)",
        );
        assert_eq!(
            format!("{}", Error::InvalidPolicyIdStubCount),
            "policy_id_stub_count must be >= 1",
        );
        assert_eq!(
            format!("{}", Error::UnexpectedEnd),
            "unexpected end of bytecode",
        );
        assert_eq!(
            format!("{}", Error::TrailingBytes),
            "trailing bytes after xpub",
        );
    }
}
