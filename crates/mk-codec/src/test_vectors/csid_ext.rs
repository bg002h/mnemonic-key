//! Canonical `mk` chunk_set_id extension vector corpus.
//!
//! Distinct file from `v0.1.json` (`super::V0_1_JSON`): the legacy corpus
//! stays byte-unchanged (it is the pinned-by-design MISMATCH half — 19/19
//! chunked vectors carry pre-0.5 randomly-drawn ids). This corpus supplies
//! the CLEAN half plus warning content, per
//! `design/SPEC_chunk_set_id_verification.md` "Vectors (R4)" and
//! `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` P0.
//!
//! Regenerated alongside `v0.1.json` by
//! `cargo run --bin gen_mk_vectors --features gen-vectors`. The pinned
//! SHA-256 over the byte sequence lives at
//! `tests/csid_ext_vectors.rs::CSID_EXT_SHA256` — a pin SEPARATE from
//! `tests/vectors.rs::V0_1_SHA256`.

/// The canonical chunk_set_id extension corpus as a UTF-8 JSON string.
///
/// `include_str!`-baked at compile time, mirroring [`super::V0_1_JSON`].
/// Unconditionally `pub` (not feature-gated) so `mk-cli` and
/// descriptor-mnemonic's `md-cli` can read the same pinned bytes in
/// later phases via `mk_codec::test_vectors::csid_ext::CSID_EXT_JSON`.
pub const CSID_EXT_JSON: &str = include_str!("csid_ext_v0.1.json");
