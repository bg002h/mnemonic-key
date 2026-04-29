# Changelog

All notable changes to `mk-codec` will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
