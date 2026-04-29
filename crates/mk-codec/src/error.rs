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

    /// xpub `public_key` bytes do not parse as a valid compressed
    /// secp256k1 point. Realistically unreachable for inputs that
    /// pass BCH verification; surfaces hand-constructed inputs.
    #[error("invalid xpub public key: {0}")]
    InvalidXpubPublicKey(String),

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
    /// correctly for every parameterized variant.
    #[test]
    fn parameterized_variants_render() {
        let cases: Vec<(Error, &str)> = vec![
            (
                Error::InvalidHrp("ms".into()),
                "invalid HRP: ms",
            ),
            (
                Error::BchUncorrectable("5 substitutions exceed long-code 4-correction limit".into()),
                "BCH uncorrectable: 5 substitutions exceed long-code 4-correction limit",
            ),
            (
                Error::UnsupportedCardType(0x05),
                "unsupported card type: 0x05",
            ),
            (
                Error::ChunkedHeaderMalformed("total_chunks = 0".into()),
                "chunked-header malformed: total_chunks = 0",
            ),
            (
                Error::InvalidXpubPublicKey("malformed compressed point".into()),
                "invalid xpub public key: malformed compressed point",
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
                Error::InvalidPathComponent("LEB128 overflow at component 3".into()),
                "invalid path component: LEB128 overflow at component 3",
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

    // ── #[ignore]-marked sad-path scaffolds (per plan §3.2.4) ──────────
    //
    // Each scaffold documents the planned decoder rejection that
    // triggers a new variant. The `#[ignore]` is removed in the phase
    // that lands the code path:
    //
    // - Phase 5 (string layer):      CrossChunkHashMismatch,
    //                                MalformedPayloadPadding,
    //                                ChunkSetIdMismatch,
    //                                ChunkedHeaderMalformed
    //
    // (Phase 4 retired the proposed FingerprintFlagMismatch variant:
    // structurally undetectable in the decoder under the closure-locked
    // wire format, since no length prefix lets the decoder distinguish
    // "flag set, fp present" from "flag unset, fp omitted." SPEC §4
    // rule 3 was reframed as an encoder-side invariant; see commit
    // log for Phase 4 review fixup.)

    #[test]
    #[ignore = "Phase 5 — string-layer reassembly"]
    fn rejects_chunked_input_with_perturbed_cross_chunk_hash() {
        // Phase 5: construct a chunked encoding, flip one byte of the
        // appended cross_chunk_hash, and assert decode(...) returns
        // Err(CrossChunkHashMismatch).
        todo!("Phase 5 — implement string-layer reassembly");
    }

    #[test]
    #[ignore = "Phase 5 — string-layer payload padding check"]
    fn rejects_singlestring_with_non_zero_pad_bits() {
        // Phase 5: construct a single-string mk1 input whose 5-bit
        // payload symbols, after BCH verification, leave non-zero pad
        // bits in the final symbol. Assert decode(...) returns
        // Err(MalformedPayloadPadding).
        todo!("Phase 5 — implement byte-align validation");
    }

    #[test]
    #[ignore = "Phase 5 — chunk-set assembly"]
    fn rejects_chunked_input_with_mismatched_chunk_set_id() {
        // Phase 5: construct two chunks with different chunk_set_id
        // values and assert decode(...) returns
        // Err(ChunkSetIdMismatch).
        todo!("Phase 5 — implement chunk reassembly");
    }

    #[test]
    #[ignore = "Phase 5 — chunked-header validation"]
    fn rejects_chunked_input_with_total_chunks_zero() {
        // Phase 5: construct a chunked input whose total_chunks field
        // is 0 (or > 32) and assert decode(...) returns
        // Err(ChunkedHeaderMalformed(_)).
        todo!("Phase 5 — implement chunked-header validation");
    }

    /// Unparameterized variants render their static message verbatim.
    #[test]
    fn static_variants_render() {
        assert_eq!(
            format!("{}", Error::ReservedBitsSet),
            "reserved bits set in bytecode header",
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
