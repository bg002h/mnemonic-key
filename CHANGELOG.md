# Changelog

All notable changes to `mk-codec` will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
