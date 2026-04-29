# Phase 4 review — bytecode-layer encoder/decoder

**Status:** DONE_WITH_CONCERNS
**Commit:** 5c5f57f
**Reviewer / Implementer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/mod.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/header.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/path.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/xpub_compact.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/encode.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/decode.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/test_helpers.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs
- /scratch/code/shibboleth/mnemonic-key/design/SPEC_mk_v0_1.md
- /scratch/code/shibboleth/mnemonic-key/bip/bip-mnemonic-key.mediawiki
- /scratch/code/shibboleth/mnemonic-key/design/IMPLEMENTATION_PLAN_mk_v0_1.md
**Role:** reviewer (code)

## Summary

Wire-format layout, header semantics, path codec, compact-73, and round-trip
match SPEC §3 and BIP §"Bytecode layer" verbatim. No blockers. Two should-address
items: `FingerprintFlagMismatch` is reachable per SPEC §4 rule #3 / BIP rule #3
but never produced by the decoder, and the planned un-ignore of the matching
scaffold in error.rs did not happen. One spec-vs-error-enum gap on
`InvalidXpubPublicKey`. A handful of test-coverage holes for §4 rules.

## Issues

### Should-address

- **SPEC §4 rule #3 / BIP rule #3 (`FingerprintFlagMismatch`) is unreachable
  in `decode_bytecode`.** decode.rs:35-40 reads the fingerprint iff the flag
  is set — verbatim. It cannot detect a hand-crafted bytecode that claims fp
  presence inconsistently. The spec rule is stated as a MUST: "decoders MUST
  reject any state where the flag and the payload disagree" (SPEC §3.4 / BIP
  L260, §"Decoder validity rules" #3). Current behavior is *consistent
  parsing* — fp-absent inputs misalign with the standard-path indicator at
  offset (1+1+4N), which surfaces as `InvalidPathIndicator` or
  `InvalidXpubVersion` or similar downstream errors rather than the named
  variant. Two acceptable resolutions:
  1. **Spec amendment** — note that under compact-73 with no length prefix,
     the rule is structurally undetectable and re-frame it as an
     encoder-side MUST only (decoder can't distinguish the two cases by
     construction). Then either retire `FingerprintFlagMismatch` from §4 or
     mark it encoder-only.
  2. **Decoder amendment** — synthesize a length check or re-derive expected
     payload length once xpub is parsed, and surface the named variant when
     the totals don't square. (More work, less elegant.)
  The "BCH-protected wire input → unreachable" framing in the request
  applies to attacker-induced bit-flips, but a spec rule that says "decoders
  MUST reject" should either be reachable in `decode_bytecode` or the spec
  should say so. The cleanest fix is option 1: amend SPEC §4 + BIP rule #3
  to "encoders MUST set the flag iff origin_fingerprint is present;
  decoders MAY rely on the flag without further consistency check (the
  payload schema is unambiguous given the flag)." This also closes the
  `decoder-error-variant-parity` audit at retirement rather than parity.

- **error.rs:188-194** — the `#[ignore]`-annotated
  `rejects_bytecode_with_fingerprint_flag_payload_disagreement` scaffold is
  still ignored after Phase 4. The plan (and the comment block at L172-186)
  explicitly says `#[ignore]` is removed in the phase that lands the code
  path; if the rule survives spec-amendment above, this scaffold needs to
  be replaced with a real test. If the rule is retired, remove the scaffold.

- **`InvalidXpubPublicKey` (error.rs:106) is not in SPEC §4 or BIP §"Decoder
  validity rules".** xpub_compact.rs:93-94 emits it on `PublicKey::from_slice`
  failure during reconstruction. Defensible as a safety net — but the
  pre-BIP `decoder-error-variant-parity` audit (FOLLOWUPS) is structured as
  *every variant maps to a spec rule and a negative vector*. As-is, this
  variant has no SPEC §4 or BIP entry and no decode.rs::tests case. Two
  resolutions: (a) add a spec rule "Has xpub public_key bytes that don't
  parse as a compressed secp256k1 point (`Error::InvalidXpubPublicKey`)" to
  SPEC §4 and BIP §"Decoder validity rules", and add a decode.rs test that
  perturbs the public_key bytes; or (b) document the variant as
  internal-only (e.g. behind `#[doc(hidden)]` or with an "internal safety
  net" doc note) and acknowledge it never fires for BCH-verified inputs.
  Recommendation: (a) — the variant exists; spec it. The variant docstring
  ("Realistically unreachable for inputs that pass BCH verification;
  surfaces hand-constructed inputs") matches a rule applicable to
  `decode_bytecode` directly, which is a public entry point.

- **decode.rs::tests gaps vs SPEC §4 rules.** Mapping audit:
  - Rule 1 UnsupportedVersion ✓ (rejects_unsupported_version)
  - Rule 2 ReservedBitsSet ✓ (rejects_reserved_bits_set)
  - Rule 3 FingerprintFlagMismatch ✗ (see above)
  - Rule 4 InvalidPolicyIdStubCount ✓ (rejects_zero_stub_count)
  - Rule 5 InvalidPathIndicator ✓ (rejects_invalid_path_indicator)
  - Rule 6 PathTooDeep ✗ — only covered in path.rs::tests, not at the
    `decode_bytecode` entry. Add a `rejects_path_too_deep_at_top_level`
    that splices an explicit-path with count=11 into the wire bytes.
  - Rule 7 InvalidPathComponent ✗ — no `decode_bytecode`-level test.
    Cover with a malformed LEB128 (e.g., 6-byte continuation overflow).
  - Rule 8 InvalidXpubVersion ✓ (rejects_invalid_xpub_version)
  - Rule 9 UnexpectedEnd ✓ (rejects_truncated_mid_stub) — adequate.
  - Rule 10 TrailingBytes ✓ (rejects_trailing_bytes)
  - (proposed) InvalidXpubPublicKey ✗ — no test at any level.

### Nit

- **xpub_compact.rs:90-92** — `child_number` falls back to
  `ChildNumber::Normal { index: 0 }` for a depth-0 path. With path-codec
  rejecting count=0 and standard-table entries having ≥3 components, the
  branch is unreachable for valid inputs. Defensible defensive code; doc
  comment at L83-85 acknowledges this. Minor: the unwrap-or branch is dead
  for the public API surface. Either drop with `unwrap()` and an
  `expect("origin_path is non-empty per spec; enforced by decode_path")`,
  or keep as-is. No action required.

- **path.rs:104** — explicit-path with `count == 0` is rejected as
  `PathTooDeep(0)` rather than a more semantically accurate "path empty"
  error. Spec §3.5 says "MUST be in 1..=10" so the rejection is correct;
  the variant name is a slight mismatch. Acceptable.

- **encode.rs:25-27** — `card.policy_id_stubs.len() > u8::MAX as usize` is
  enforceable but the spec says practical limit is set by chunk-payload
  capacity. At 256 stubs (>1KB just for stubs), no chunk envelope holds
  this. The `> 255` check is correct as wire-format guard; acceptable.

- **path.rs:62-71** `lookup_path` — structural comparison is correct for
  every entry in the table. Each entry parses to a unique `DerivationPath`
  with hardened components; no two entries share a structural form. The
  `m/`-prefix Display pitfall justification holds. No semantic distinction
  between a user's `DerivationPath` and a structurally-equal table entry —
  `DerivationPath` is just `Vec<ChildNumber>`. Encoder-side, the round-trip
  is byte-exact when a path matches a table entry. No issue.

## Confirmations

- Bytecode header bit semantics match SPEC §3.1 verbatim: bit 2 fingerprint
  flag, bits 0/1/3 reserved, version field 7-4. Valid header bytes 0x00 and
  0x04 verified in tests; 0x10, 0xF0, bit-0/1/3-set, and combinations all
  rejected.
- Payload field order in encode.rs matches SPEC §3.2 / BIP L232-238:
  header → stub_count → stubs → fp (conditional) → path → xpub_compact.
  Decoder reads in the same order. Fixture
  `encodes_typical_1stub_card_to_84_bytes` verifies byte-exact layout
  (asserts wire[0]==0x04, wire[1]==1, wire[2..6]==stub, wire[6..10]==fp,
  wire[10]==0x05).
- Standard-table contents (path.rs:29-45): 14 entries; mainnet 0x01-0x07,
  testnet 0x11-0x15, 0x17. 0x16 absent (reserved-pending-md1). Verified
  against SPEC §3.5 table and BIP §"Origin path encoding". `0x16` decode
  rejection covered (path.rs:262-270, decode.rs:172-182).
- Explicit-path encoding: 0xFE indicator + 1-byte count + LEB128 u32
  components per SPEC §3.5 / BIP L272-278. Hardened bit is preserved
  via the bit-31 carry into LEB128 (path.rs:84). 5-byte worst-case
  matches §3.5 component-byte sizing. Verified by
  `round_trip_explicit_path_all_hardened` (22 bytes for 4 hardened
  components).
- Path-component cap = 10 enforced both ways (path.rs:104, MAX_PATH_COMPONENTS
  in consts.rs); rejected at count=11 (rejects_path_too_deep).
- Compact-73 layout: 4 + 4 + 32 + 33 = 73 bytes. Encoder
  (xpub_compact.rs:106-111) and decoder (xpub_compact.rs:114-132) both
  match SPEC §3.6 byte order. `XPUB_COMPACT_BYTES` const sanity-checked
  in consts.rs.
- Compact-73 reconstruction: depth = component_count, child_number =
  last_component (xpub_compact.rs:88-92) per SPEC §3.6. `child_number`
  preserves the hardened-bit encoding because `ChildNumber` carries it.
  Verified in `round_trip_full_xpub_depth_4` (depth 4 reconstructed,
  child_number matches).
- LEB128 round-trips correctly for 0, 127, 128, 0x80000000 (path.rs:302-319
  test). I traced 0, 127, 128, 0x7FFFFFFF, 0x80000000, 0xFFFFFFFF
  end-to-end:
  - 0 → [0x00] → 0
  - 127 → [0x7F] → 127
  - 128 → [0x80, 0x01] → 128
  - 0x7FFFFFFF → 5 bytes [0xFF, 0xFF, 0xFF, 0xFF, 0x07] → 0x7FFFFFFF
  - 0x80000000 → 5 bytes [0x80, 0x80, 0x80, 0x80, 0x08] → 0x80000000
    (verified in test)
  - 0xFFFFFFFF → 5 bytes [0xFF, 0xFF, 0xFF, 0xFF, 0x0F] → 0xFFFFFFFF
- LEB128 6-byte+ overflow is rejected: at byte 5 with continuation set,
  shift becomes 35, `shift >= 35` guard returns InvalidPathComponent.
  Plus `result > u32::MAX` guard catches the case where byte 5 has bits
  32-34 set without continuation. Both branches reachable; both correct.
- All 43 in-crate tests pass; integration round_trip is gated by Phase 5.
- Fingerprint flag is correctly synthesized from `origin_fingerprint.is_some()`
  on encode (encode.rs:31) and the inverse on decode (decode.rs:35).
  Privacy-preserving mode (no fp) round-trips end-to-end
  (`encodes_card_without_fingerprint_to_80_bytes`, `round_trip_3stubs_no_fp`).
- Deterministic encoder verified (encode.rs:100-105). KeyCard equality
  (PartialEq) holds across round-trip including reconstructed xpub.

## Open observations

- **Test-fixture realism.** `synthetic_xpub` (test_helpers.rs:19-37) builds
  `Xpub` with deterministic `chain_code = [0x55; 32]`, `parent_fingerprint =
  [0xAA, 0xBB, 0xCC, 0xDD]`, `public_key = SecretKey::from_slice(&[1u8;
  32])` derived. All 32-byte and 33-byte fields are *non-zero* and *all-bits
  varied* enough that a wire-format byte-order regression (e.g., swapping
  parent_fingerprint with the chain_code's first 4 bytes) would surface as
  byte mismatch. The synthetic data is sufficient for *byte-layout drift*
  detection. It is *not* sufficient for cross-format verification (a real
  BIP 32 xpub against, say, the Trezor xpub-test-vector page); that
  validation lands at Phase 6 vector-corpus generation. No action.

- **Edge cases checked.**
  - depth-0 path: not constructible from path codec (count=0 rejected,
    standard table all ≥3 components). Reconstructor's fallback to
    `ChildNumber::Normal { index: 0 }` is unreachable from the public
    API.
  - All-non-hardened explicit path: `m/0/1/2` round-trips
    (round_trip_explicit_path_simple). The encoder picks explicit-path
    correctly because no standard-table entry is all-non-hardened.
  - Trailing bytes: rejected (rejects_trailing_bytes).
  - UnexpectedEnd boundary: rejected mid-stub (rejects_truncated_mid_stub).
  - stub_count = 0xFF (max single-byte): not directly tested but
    structurally OK; the `> u8::MAX` guard is a no-op since Vec.len() ≤
    255 fits in u8 by construction. Edge worth a fixture: 255 stubs
    encodes to 1+1+255*4+... = 1023+ B, large enough to require chunked
    string-layer envelope. Optional Phase 5 fixture, not Phase 4 issue.

- **Plan deviations.** Phase 4 added `InvalidXpubPublicKey` not in the
  Phase 3 Error overhaul plan. Defensible (real failure mode at
  PublicKey::from_slice), but should be back-propagated to SPEC §4, BIP
  §"Decoder validity rules", and FOLLOWUPS `decoder-error-variant-parity`
  bookkeeping. See above.

- **Cross-format pattern compliance (D-14).** Header bit-2 fp-flag
  semantics match md1's pattern at the closure-spec level. Confirmed via
  SPEC §3.1 closing paragraph. The shared header-parsing helper
  extraction (Q-9 trigger) is unblocked by this implementation.
