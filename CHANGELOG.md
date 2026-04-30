# Changelog

All notable changes to `mk-codec` will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] — 2026-04-30

Doc-only patch. Closes the four deferred suggestions from the v0.2.0
Phase 2-4 opus review (`design/agent-reports/v0-2-0-phase-2-4-review-fd6a407.md`).
**Wire format and corpus byte-identical to v0.2.0**; SHA pin
unchanged (`ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff`).
No code change; no test change.

### Added

- BIP §"Origin path encoding" Case A — full path-dictionary table
  inline (14 rows mirroring md1's `Tag::SharedPath`). Replaces the
  prose-only enumeration with a single source of truth that's harder
  to drift out of sync as future entries land. (Phase 4 / S-4)

### Changed

- CHANGELOG `[0.2.0]` Notes — backfilled the missing
  cross-implementation SHA-pin migration pointer that should have
  been in the original v0.2.0 release notes (parallel to v0.1.1's
  pattern). v0.2.0 → v0.1.1 cross-impl migrants now have the
  explicit pointer they need. (Phase 1 / S-1)
- `crates/mk-codec/tests/vectors.rs::VECTOR_FILE` — added a comment
  documenting the filename-vs-family-token convention: filename is
  intentionally stable across minor-bump family-token rolls; the
  corpus's `family_token` field carries the rolling version per Q-10.
  (Phase 2 / S-2)
- `crates/mk-codec/src/bin/gen_mk_vectors.rs` module rustdoc — dropped
  the misleading "v0.1 vector corpus" version specifier; the binary
  generates whatever family `GENERATOR_FAMILY` names. (Phase 3 / S-3)

### Notes

- Patch release: no wire-format change, no API change, no test change,
  no corpus change. Pure doc/comment polish.
- Cross-implementations need no migration work for v0.2.1; existing
  v0.2.0 conformance pins (V0_1_SHA256, family_token) remain valid.
- The CHANGELOG `[0.2.0]` Notes amendment is a retroactive backfill;
  the underlying v0.2.0 release artifact (tag, GitHub release) is
  unchanged. Future readers see the migration pointer in the [0.2.0]
  entry as if it had always been there, with the dated provenance
  noted in the entry itself.

## [0.2.0] — 2026-04-30

Wire-additive minor bump: closes the BIP 48 testnet nested-segwit
multisig path-dictionary gap on the mk1 side after md-codec v0.9.0
closed it on the md1 side. v0.1.x decoders reject v0.2.0-emitted
strings carrying indicator `0x16`; v0.2.0+ decoders accept and resolve
to `m/48'/1'/0'/1'`. All other v0.1.x string encodings round-trip
byte-identical through v0.2.0.

### Added

- Path-dictionary indicator `0x16` for `m/48'/1'/0'/1'` (BIP 48 testnet
  nested-segwit multisig). Closes `md-path-dictionary-0x16-gap`
  (FOLLOWUPS) and brings mk1's standard-table dictionary to its full
  14 entries, matching md-codec v0.9.0+'s table byte-for-byte.
- `V18_bip48_nested_segwit_testnet_1_stub_with_fp` — corpus vector
  exercising the new indicator. 18 clean + 22 negative = 40 vectors total.
- `crates/mk-codec/tests/error_coverage.rs` — strum-driven exhaustiveness
  gate for negative vectors. Mirrors md-codec's `error_coverage.rs`
  pattern: a hand-written `ErrorVariantName` mirror enum with
  `#[derive(strum::EnumIter)]` lets the test iterate every variant
  without requiring the source `Error` to be `EnumIter`-compatible
  (parameterized variants make direct iteration awkward). Two checks:
  every variant has a corpus vector or an explicit exemption (forward),
  and every negative vector's `expected_error` maps to a known variant
  (reverse).
- `strum = { version = "0.26", features = ["derive"] }` as a
  dev-dependency. md-codec uses the same pin.

### Changed

- `GENERATOR_FAMILY` rolls `"mk-codec 0.1"` → `"mk-codec 0.2"` per
  closure Q-10 (minor-version bumps roll the family token; patches
  don't). Test-vector corpora regenerate accordingly; v0.1.x corpora
  remain valid for v0.1.x consumers but the v0.2.0 corpus is the new
  conformance reference for v0.2+ implementations.
- BIP §"Bytecode header" / SPEC §3.1 — added a paragraph clarifying
  that the bit-allocation **shape** is shared between mk1 and md1
  (4-bit version + 4 flag/reserved bits) and **bit-2 semantics** are
  shared (optional-fingerprint flag), while specific allocations of
  bits 0, 1, 3 diverge from bit 3 onward. md-codec v0.10.0 reclaimed
  bit 3 as the OriginPaths flag on the md1 side; mk1's bit 3 remains
  reserved.
- `tests/vectors/v0.1.json` SHA-256 pin rolls to
  `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff`.

### Removed

- `tests/vectors.rs::every_error_variant_has_negative_vector` runtime
  substring gate. Replaced by the strum-driven gate in
  `tests/error_coverage.rs`. Pointer comment in `vectors.rs` directs
  readers to the new location.

### Resolved (FOLLOWUPS)

- `error-variant-exhaustiveness-gate-strum` (v0.2-nice-to-have) —
  strum-driven gate landed in Phase 1.
- `md-path-dictionary-0x16-gap` cross-update — entry was already
  closed on the md-codec side in v0.9.0; mk-codec v0.2.0 adds the
  encoder/decoder support and corpus vector.

### Notes

- Wire-additive change: v0.1.x decoders reject v0.2.0-emitted strings
  carrying `0x16` with `Error::InvalidPathIndicator(0x16)`. Cross-
  implementations consuming v0.2.0 vectors MUST update to a v0.2+
  decoder.
- All v0.1.0 / v0.1.1 string encodings round-trip byte-identical through
  the v0.2.0 decoder. Backward compatibility is one-way: v0.2.0 reads
  v0.1.x; v0.1.x doesn't read v0.2.0 if `0x16` is in play.
- Cross-implementations validating against the v0.1.x corpus need to
  update their SHA-256 pin to match the regenerated v0.2.0 corpus
  (`ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff`)
  and expect 18 clean + 22 negative = 40 vectors (was 17 + 22 = 39).
  The family token rolls under the Q-10 minor-bump convention; v0.1.x
  corpora remain valid for v0.1.x consumers. (Backfilled in v0.2.1; see
  `[0.2.1]` below for details — the original v0.2.0 release notes
  omitted this migration pointer.)
- The closure-design's path-dictionary-mirror-stewardship contract
  (mk1 inherits md1's table) auto-extended mk1's coverage when md1
  v0.9.0 added the indicator; v0.2.0 makes that auto-extension
  observable in the encoder/decoder + corpus.
- The BIP §"Bytecode header" bit-3 footnote is doc-only; mk1's bit 3
  remains reserved-must-be-zero. No mk1 wire-format change beyond
  the path-indicator addition.

## [0.1.1] — 2026-04-29

Patch release: v0.1-nice-to-have backlog clearance + vector corpus
expansion. **Wire format byte-identical to v0.1.0**; existing v0.1.0
strings round-trip unchanged through the v0.1.1 decoder.

### Added

- `Error::MixedHeaderTypes` variant for header-type disagreement across
  a multi-string input (forward direction: `[SingleString, Chunked]`;
  reverse direction: `[Chunked, ..., SingleString, ...]`). Previously
  surfaced as `Error::ChunkedHeaderMalformed` with overloaded message
  text; precise discrimination introduced in v0.1.1.
- 9 new clean vectors (V9..V17) covering all path-dictionary entries
  except 0x16 (BIP 48 testnet nested-segwit) which remains blocked on
  the cross-repo `md-path-dictionary-0x16-gap` resolution.
- 22 negative vectors (N1..N21, N23) — one per `Error` variant
  reachable from `decode`'s string-input path. Schema-2 vectors carry
  a new `expected_error` field with the byte-exact `Error::Display`
  rendering. `Error::CardPayloadTooLarge` is encoder-only and exempt
  from corpus coverage; documented in the exhaustiveness gate.
- Vector-corpus schema bumped 1 → 2. The `expected_error` field is
  emitted on every vector (`null` for clean, string for negative); the
  always-emit rule preserves byte-determinism. Schema-1 corpora remain
  parseable by the v0.1.1 harness if a future contract relaxes the
  schema-version pin.
- `every_error_variant_has_negative_vector` integration test asserts
  every reachable `Error` variant has at least one negative vector.
  (Implementation note: runtime substring gate; a strum-driven
  compile-time variant — recorded as `error-variant-exhaustiveness-gate-strum`
  in `design/FOLLOWUPS.md` at v0.2-nice-to-have tier.)

### Changed

- `decode_rejects_perturbed_cross_chunk_hash` (now
  `decode_rejects_5_symbol_burst_in_last_chunk_data_part`) hardened to
  perturb at the 5-bit-symbol layer past the 8-symbol chunked header,
  with a 5-symbol burst that exceeds BCH(108,93,8) / BCH(93,80,8)
  `t = 4` correction radius. Accepting set widened to
  `{CrossChunkHashMismatch, BchUncorrectable}`. Removes the silent-
  un-flip risk in the v0.1.0 fixture and ties the test invariant
  tightly to the BCH-distance argument.
- `pipeline::decode` rustdoc updated to cite `Error::MixedHeaderTypes`
  for the mixed-header rejection (was `ChunkedHeaderMalformed`).

### Resolved (FOLLOWUPS)

- `cross-chunk-hash-test-fixture-stability` — hardened test, see Changed.
- `pipeline-decode-mixed-header-error-naming` — `MixedHeaderTypes` added.
- `vector-corpus-dictionary-coverage` — 9 new path-dictionary vectors.
- `decoder-error-variant-parity` — 22 negative vectors per `Error` variant.
- `encode-with-chunk-set-id-singlestring-silent-ignore` — closed as
  `wont-fix` per SPEC §2.4 (SingleString unreachable for v0.1
  conforming KeyCards). Sequencing requirement recorded for any future
  smaller-bytecode wire-format extension.

### Notes

- Cross-implementations validating against the v0.1.0 corpus need to
  update their `V0_1_SHA256` pin to match the expanded v0.1.1 corpus
  and migrate to schema 2 to consume negative vectors. Existing v0.1.0
  clean-vector encodings remain byte-identical in v0.1.1 (verified).
- `Error::MixedHeaderTypes` is `#[non_exhaustive]`-safe; downstream
  exhaustive-match consumers won't break. Text-match consumers of the
  message strings would observe a behavior change; CHANGELOG calls
  this out for migration awareness.
- Pre-BIP-submission audit gates remain — see `design/FOLLOWUPS.md` at
  tier `pre-bip-submission`. Notable open items: NUMS structural audit,
  formal HRP collision check, BIP cross-reference completeness, and
  the cross-repo `chunk-set-id-rename` in md-codec (sequencing
  prerequisite for mk1's BIP submission).
- Eventual `mc-codex32` shared-crate extraction (closure Q-9) remains
  deferred; BCH primitives are still forked-not-shared.

## [0.1.0] — 2026-04-29

First reference implementation of the **Mnemonic Key (MK)** backup format.

### Added

- Working `KeyCard` ↔ `Vec<String>` round-trip via `mk_codec::encode` /
  `mk_codec::encode_with_chunk_set_id` / `mk_codec::decode`. Encoder
  draws a fresh 20-bit `chunk_set_id` from the system CSPRNG by default;
  callers wanting deterministic output (vector regeneration, conformance
  tests) use `encode_with_chunk_set_id` to pin the value.
- BCH error-correction layer (`BCH(93,80,8)` regular code +
  `BCH(108,93,8)` long code) forked from sibling `md-codec` per
  [`design/DECISIONS.md`](design/DECISIONS.md) D-13. Polynomial /
  HRP-mixing is shared with md1; only HRP (`"mk"`) and the NUMS-derived
  target residues `MK_REGULAR_CONST` / `MK_LONG_CONST` differ.
- 5-bit-symbol-aligned string-layer header (closure Q-5):
  `SingleString` (2 symbols) and `Chunked` (8 symbols carrying
  `version + type + chunk_set_id + total_chunks + chunk_index`).
- Compact-73 xpub form (closure Q-7): xpub `version + parent_fingerprint +
  chain_code + public_key` written verbatim on the wire; `depth` and
  `child_number` are reconstructed at decode time from the
  `origin_path` field (no on-wire redundancy).
- Optional `origin_fingerprint` field via bytecode-header bit 2
  (closure Q-8 privacy-preserving mode).
- Standard-table path dictionary mirroring md1's `Tag::SharedPath`
  byte-for-byte (BIP 44 / 49 / 84 / 86 / 48-segwit / 48-nested / 87 +
  testnet variants), with a `0xFE` explicit-path escape carrying up
  to 10 LEB128 components (closure Q-3 cap).
- Cross-chunk integrity hash: `SHA-256(canonical_bytecode)[0..4]`
  appended pre-split, verified at reassembly.
- Initial vector corpus (8 hand-curated fixtures) at
  `crates/mk-codec/tests/vectors/v0.1.json`, anchored under family
  token `"mk-codec 0.1"` with a SHA-256-pinned conformance gate.
  Cross-implementations validate by matching this file's SHA plus
  every vector's round-trip.
- Optional `gen-vectors` Cargo feature exposing the
  `gen_mk_vectors` regenerator binary.

### Notes

- Wire format finalized per the v0.1 closure design (Q-1..Q-10 closed
  in [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md)).
- `SingleString` chunk-type variant is wire-defined for forward
  compatibility but **unreachable** for v0.1 conforming encoders —
  the smallest valid bytecode (1+1+4+1+73 = 80 bytes) already
  exceeds the 56-byte single-string capacity. Every v0.1 mk1 KeyCard
  encodes as a chunked card.
- Pre-BIP-submission audit items remain — see
  [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) at tier
  `pre-bip-submission`. Notable: NUMS structural audit, formal HRP
  collision check, decoder-error-variant-parity vectors, BIP
  cross-reference completeness, vector-corpus dictionary expansion
  beyond the 8 v0.1 fixtures.
- Eventual `mc-codex32` shared-crate extraction (closure Q-9) is
  deferred until both md-codec and mk-codec reach v1.0 with
  cross-validated conformance vectors. Until then, BCH primitives are
  forked-not-shared; both implementations carry their own copy.
