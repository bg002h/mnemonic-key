# Phase 3 review — consts module + Error overhaul (commit 4a80d34)

**Status:** DONE
**Commit:** 4a80d34
**Reviewer / Implementer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/consts.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/lib.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs`
- `/scratch/code/shibboleth/mnemonic-key/design/SPEC_mk_v0_1.md` §2.4, §3.x, §4
- `/scratch/code/shibboleth/mnemonic-key/bip/bip-mnemonic-key.mediawiki` §"Length envelope", §"Decoder validity rules"
- `/scratch/code/shibboleth/mnemonic-key/design/IMPLEMENTATION_PLAN_mk_v0_1.md` Phase 3
- `/scratch/code/shibboleth/mnemonic-key/docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`

**Role:** reviewer (code)

## Summary

No blockers. Constants are byte-correct, every SPEC §4 / BIP rule maps to a unique Error variant, `XpubDepthMismatch` is removed, `PathTooDeep` carries the cap-10 message, both `Error` and `KeyCard` are `#[non_exhaustive]`, re-exports are explicit, and tests pass (8/0/0 in lib + 1 ignored round-trip). Two should-address items below; one is a TDD-discipline drift from the plan, one is a meaningful gap in Display coverage.

## Issues

### should-address — Phase 3 plan calls for `#[ignore]`-marked sad-path test scaffolds; commit ships none
Plan §"Task 3.2 — Step 3.2.4" (`IMPLEMENTATION_PLAN_mk_v0_1.md` line 477–479) says:

> For each new variant, add an `#[ignore]`-marked test in the relevant module that documents the expected reject case. The `#[ignore]` is removed in the phase that lands the code path.

The commit ships zero `#[ignore]`-marked sad-path placeholders. `cargo test` reports `0 ignored` in lib (only the existing `tests/round_trip.rs` placeholder is ignored). This is the TDD discipline the plan's overall strategy section (line 9) explicitly invokes — "tests start `#[ignore]`-marked or build-failing and become passing as the impl substeps complete." Phase 4/5 inherits the slack: when those phases land, the implementer has to write the test from scratch *and* the impl in the same commit, defeating the test-first cadence. Resolution: add `#[ignore = "Phase 4"]` / `#[ignore = "Phase 5"]`-marked stub tests in `error.rs::tests` (or a `tests/` file) — at least one per new variant: `CrossChunkHashMismatch`, `FingerprintFlagMismatch`, `MalformedPayloadPadding`, `ChunkSetIdMismatch`, `ChunkedHeaderMalformed`. Each can be a one-liner `panic!("rejection vector pending Phase X")` whose presence forces the Phase 4/5 implementer to remove the ignore and supply the vector. Tracking-wise this also gives the Phase 8 reconciliation pass a checklist.

### should-address — `static_variants_render` test misses three variants; coverage isn't exhaustive
`error.rs:155–189`. The test name suggests it covers all unparameterized variants, but it omits `BchUncorrectable` (well, `BchUncorrectable` is parameterized — fine) and skips `parameterized_variants_render` does not cover `InvalidPathComponent(String)` or `ChunkedHeaderMalformed(String)` or `BchUncorrectable(String)`. Specifically: `BchUncorrectable`, `InvalidPathComponent`, and `ChunkedHeaderMalformed` are parameterized variants (each takes a `String`) and none appear in the parameterized test (`error.rs:122–152`). 17 variants total; 14 covered (6 parameterized + 8 static). Resolution: add the three missing parameterized cases to `parameterized_variants_render`. Drift detection for Display strings is one of the few things these tests can plausibly catch; partial coverage understates the variant inventory and lets a future rename of an uncovered variant's `#[error("...")]` slip through.

### nit — `consts.rs` exports `CROSS_CHUNK_HASH_BYTES` but the plan didn't enumerate it
`consts.rs:45` and `lib.rs:44` add `CROSS_CHUNK_HASH_BYTES`, `XPUB_COMPACT_BYTES`, `POLICY_ID_STUB_BYTES`, `ORIGIN_FINGERPRINT_BYTES` beyond the plan's listed constants (plan §3.3.1 line 487–499 only listed the original twelve). All four are SPEC-justified and useful for Phase 4/5 codecs; the deviation is a beneficial superset. Worth noting in the FOLLOWUPS log so plan and code don't drift in opposite directions later.

### nit — `nums_string_differs_from_md1` is the only md1-vs-mk1 cross-check
`consts.rs:94`. Useful, but doesn't catch the case where `NUMS_DOMAIN` is changed to some other non-md1 string. Not worth a fix; the `nums_constants_reproduce_from_domain` covers drift between domain and constants, which is the only real risk class.

## Confirmations

- **NUMS reproducibility verified independently.** `SHA-256(b"shibbolethnumskey")` = `83121afc88397d2e4e7f2ba3502f9755...`; top-128-bits BE u128 = `0x83121afc88397d2e4e7f2ba3502f9755`; `>> 63` = `0x1062435f91072fa5c` (matches `MK_REGULAR_CONST` at `consts.rs:18`); `>> 53` = `0x41890d7e441cbe97273` (matches `MK_LONG_CONST` at `consts.rs:21`). The Rust sanity test (`consts.rs:71–91`) does the same staging and is genuinely drift-detecting, not a tautology. The `>> 63` / `>> 53` shift counts are correct for top-65 / top-75 bits of a leading-128-bit integer.
- **Capacity constants correct.** 48 / 56 / 45 / 53 match SPEC §2.4 (line 76–82) and BIP §"Length envelope" (lines 131–132). MAX_CHUNKS=32 matches both. CROSS_CHUNK_HASH_BYTES=4 matches SPEC §2.6 (line 128) and BIP. XPUB_COMPACT_BYTES=73 matches SPEC §3.6 (4+4+32+33). POLICY_ID_STUB_BYTES=4 matches SPEC §3.3 (closure Q-2). ORIGIN_FINGERPRINT_BYTES=4 matches SPEC §3.4. MAX_PATH_COMPONENTS=10 matches closure Q-3 / SPEC §3.5.
- **Error variant ↔ rule parity verified.** SPEC §4 (lines 268–284) enumerates 14 reject conditions (10 bytecode + 4 string-layer); BIP adds a fifth string-layer rule (`UnsupportedCardType`, line 331). Variant inventory (`error.rs`):
  - String-layer (7): `InvalidHrp`, `BchUncorrectable`, `UnsupportedCardType`, `MalformedPayloadPadding`, `ChunkSetIdMismatch`, `ChunkedHeaderMalformed`, `CrossChunkHashMismatch`.
  - Bytecode-layer (10): `UnsupportedVersion`, `ReservedBitsSet`, `FingerprintFlagMismatch`, `InvalidPolicyIdStubCount`, `InvalidPathIndicator`, `PathTooDeep`, `InvalidPathComponent`, `InvalidXpubVersion`, `UnexpectedEnd`, `TrailingBytes`.
  - 17 variants total. 10/10 SPEC §4 bytecode rules ↔ unique variants; 5/5 string-layer rules ↔ unique variants. `InvalidHrp` and `BchUncorrectable` are pre-§4 plumbing (HRP rejection + BCH-uncorrectable) — not in the §4 enumeration but obviously required. No orphans either direction.
- **`XpubDepthMismatch` confirmed removed.** No occurrence in `error.rs` (or anywhere in the source tree). Spec §4 footer (line 286) and closure Q-7 ripple match.
- **`PathTooDeep` cap message updated.** `error.rs:90` reads `"path too deep: {0} components (max 10)"` — closure Q-3 lock is reflected.
- **`#[non_exhaustive]` discipline.** `error.rs:18` for `Error`, `key_card.rs:27` for `KeyCard`. Both correct.
- **Re-export hygiene.** `lib.rs:41–57` lists 15 consts explicitly (no glob, no ellipsis — closes plan-review-1 nit 7). Plus `error::{Error, Result}` and `key_card::{decode, encode, KeyCard}`. No private items leaked; `consts::tests` is `#[cfg(test)]`-gated and not re-exported. Module declarations match.
- **`FingerprintFlagMismatch` naming.** Lands as renamed per plan-review-1 nit 8 (was `FingerprintFlagPayloadDisagreement`).
- **Pre-existing scaffold compatibility.** `key_card.rs:39` keeps `origin_fingerprint: Fingerprint` (not Optional) — Phase 4 changes it, as the commit message and plan §4 note. `types_compile` test passes (lib summary). `tests/round_trip.rs` still `#[ignore]`-marked (visible in test output: `ignored, mk-codec encode/decode not yet implemented`).

## Open observations

- The Phase 3 plan re-export list omits four byte-size consts the implementation added (`CROSS_CHUNK_HASH_BYTES`, `XPUB_COMPACT_BYTES`, `POLICY_ID_STUB_BYTES`, `ORIGIN_FINGERPRINT_BYTES`). Beneficial superset; consider patching the plan or noting in `FOLLOWUPS.md` so the divergence is intentional rather than accidental drift.
- `lib.rs:35` carries `#![cfg_attr(not(test), deny(missing_docs))]`. Every const and Error variant in the new code has a `///` doc comment, so this gate is honored. Future variants/consts must keep this discipline.
- `consts.rs` uses `bitcoin::hashes::sha256::Hash` for the sanity test; the plan sketch (line 408) referenced `sha2::Sha256`. The substitution is justified — it avoids adding a new dev-dependency since `bitcoin` is already a dependency. Behavior is equivalent.
- `parameterized_variants_render` covers `InvalidPathIndicator(0x16)` — ironically the byte called out in SPEC §3.5 as the reserved-pending-md1 row. Nice anchor.
