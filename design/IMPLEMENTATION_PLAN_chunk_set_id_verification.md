# IMPLEMENTATION PLAN — chunk_set_id recompute-and-report

**Status: DRAFT for R0.** Implements `design/SPEC_chunk_set_id_verification.md`
@ `fcf3971` (GREEN across four lenses). TDD, phased, test-first.

**Baseline revisions (staleness anchor — re-validate before dispatching each
phase's implementer; a GREEN earned against a moved tree is stale):**
mnemonic-key `7ef32f7`, descriptor-mnemonic `044e33d4`, seedhammer `5f02773c`.
Note: descriptor-mnemonic moved since the recon (`7eca44b6`→`044e33d4`) but the
`crates/md-cli/src/seat/` path did NOT change (`git log 7eca44b6..044e33d4 --
seat/` empty), so P3's citations hold; re-confirm at dispatch.

## Machine-verified facts this plan rests on (do not re-derive)

- `mk_codec::derive_chunk_set_id` — public (`crates/mk-codec/src/lib.rs:52`).
- `mk_codec::encode_bytecode(&KeyCard) -> Result<Vec<u8>>` — public
  (`crates/mk-codec/src/bytecode/mod.rs:28`, def `bytecode/encode.rs:24`).
- The operand is `derive_chunk_set_id(encode_bytecode(decoded_card))` (spec
  "The comparison"); both public, so **no mk-codec change, no publish**.
- Shared mk1 intake: `crates/mk-cli/src/cmd/mod.rs:212 read_mk1_strings`
  (six verbs: decode/inspect/verify/repair/derive/address).
- Seat grouping discards headers today: `GroupId` carries only the id
  (`descriptor-mnemonic seat/input.rs:44-60`); `{id:05x}` rendering at `:59`.
- Vector generator: `cargo run -p mk-codec --bin gen_mk_vectors --features
  gen-vectors` (`crates/mk-codec/Cargo.toml:26`); corpus baked at
  `crates/mk-codec/src/test_vectors/v0.1.json`.
- Legacy corpus is pinned-by-design MISMATCH half (19/19 declared≠derived);
  the extension corpus is a NEW file — v0.1 stays byte-identical.

## Build gate

This plan carries almost no standalone algorithm — it wires two public
functions into existing surfaces and edits message strings. There is no
extractable `rust` block that assembles into a scratch crate, so the
mnemonic-engrave `scripts/plan-build-gate.sh` pattern does not apply here.
The gate is instead each phase's own `cargo nextest run --locked -p <crate>`
plus the corpus round-trip. State this in every review brief so reviewer
budget goes to design, not to "does it compile".

## Phasing (each phase: RED tests first → impl → `cargo nextest run --locked`
green + 0C/0I re-review before advancing; parallel between phases forbidden,
one implementer per phase per the tight-implementation rule)

### P0 — the extension vector corpus (foundation; everything asserts against it)

The corpus is the executable anchor (R4); it must exist before the surfaces
that assert on it. Extend `gen_mk_vectors` to emit a SECOND file
`crates/mk-codec/src/test_vectors/csid_ext_v0.1.json` with per-row:
`canonical_bytecode_hex`, mk1 string set, `declared_csid`, `derived_csid`,
`expect_mismatch_warning`, `warning_text`. Required rows (spec Vectors §):
- clean twins of three legacy shapes (derived==declared);
- the walk's three seed cards: plate A `1b1ba/1b1ba`, plate B `ef12f/ef12f`,
  pinned `12345/ef12f`;
- **a row pinning the current 14-entry `STANDARD_PATHS` table** (r4 L1-I1) so
  a future entry trips this test, not a field warning;
- **at least one row whose derived id is `< 0x10000`** (r4 L1-I2) exercising
  `{:05x}` zero-padding.
Tests: a Rust test reads the file and asserts every `derived_csid` reproduces
`derive_chunk_set_id(encode_bytecode(decode(strings)))`; SHA-pin the new file
separately from `V0_1_SHA256`; assert v0.1.json byte-unchanged.
Gate note: this is the ONE phase whose "test can fail" claim needs the
mutation check — corrupt a row's `derived_csid` and the reader test must fail.

### P1 — mk-cli read side: the R2 warning on all six verbs + inspect print

RED: golden-stderr tests (one per verb: decode/inspect/verify/derive/address,
plus repair deferred to P2) asserting the warning fires on a pinned-card row
and is ABSENT on its clean twin; exit code unchanged; `{:05x}` rendering; the
W13 remedy wording; content asserts the corpus `warning_text`.
IMPL: compute `derived` at the shared decode point feeding
`read_mk1_strings`' consumers (or each verb's decode call), emit one warning
per mismatching group in existing group order (r4 L2-M2). `mk inspect` also
prints the stamped id unconditionally (r1 M4). `mk verify --json` gains the
additive `chunk_set_id` object, integer `schema_version` held at 1 (L2-I3);
NO other JSON envelope changes.
MUTATION: delete the mk-cli comparison → P1 rows fail, with evidence the line
ran (r1 M2/r2 M2).

### P2 — mk-cli write side + repair blessed path

RED: `mk encode --chunk-set-id` mint warning (W13/W14 wording, drop-the-flag +
anti-transcription clauses); assert exit 0 and strings still mint. Repair:
DAMAGED pinned card → exit 5, warning + mint-time clause fire; undamaged
pinned (exit 0) and single-chunk Candidate (exit 5) → SILENT (r4 L2-I1).
IMPL: mint path derives the id itself via the public pair (pinned arm skips it,
r1 M5). Repair plumbs the blessed `Ok(card)` out (r4 L2-N1 — no second decode);
warning carries the mint-time clause (r4 L2-I2). `repair --json` UNCHANGED
(D27 byte-match contract — r4 L2-I3).

### P3 — descriptor-mnemonic seat path: warning + R5 refusal rewrite

RED: seat warning after clean reassembly with a pinned card (contract 6); the
four-situation classification (r2 C3 — arm 3 unconditioned/total), each with a
vector incl. the mixed-halves cross-chunk case; the retired "re-mint one of
them" string appears in NO test; the W15/W16 elements (piece-number evidence,
order-doesn't-matter, one-card-at-a-time, named `mk inspect` id-check,
cards-never-plates, human-sentence-first + labeled codec line).
IMPL: `group_key_of` must RETAIN per-chunk `chunk_index`/`total_chunks`
(today discarded — r1 C3) to drive the classifier; add the warning at the
post-reassembly success point in `md descriptor`/`md address`. Update the
enumerated churn sites (spec contract 7: `seat/input.rs:310-313` assert,
`tests/seating_vectors.rs:845-846`, module doc `:1-25`/`:106`, doc `:107`).
MUTATION: delete the md-cli comparison → P3 rows fail.

### P4 — seedhammer fork: derivation-parity test only

RED+IMPL: a Go unit test asserting `top20(sha256(bytecode))` reproduces the
extension corpus's clean pinned rows (hand-carried like existing
`parityVectors`). NO device UI, NO JSON ingestion (both post-cycle followups).
Rust-primary: this is convergence, Rust already leads.

## Acceptance (whole-cycle, per spec Acceptance §)

- `cargo nextest run --locked` green in mnemonic-key and descriptor-mnemonic;
  `go test ./mk/` green in the fork incl. the new parity test.
- Per-surface golden rows fire on mismatch / silent on clean twins; repair
  scoped to its blessed path (damaged supply); the two named guarantees
  (`an_explicit_chunk_set_id_still_wins`, `canonical_payload_is_chunk_set_id_invariant`)
  byte-unchanged; v0.1.json byte-unchanged.
- Per-surface mutation gates (mk-cli, md-cli, Go) each fail their rows with
  the mutated line proven to have RUN.
- Whole-diff independent review (R0 = plan correctness; this catches
  implementation-introduced regressions TDD misses) before ship.

## Out of scope (post-cycle burndown, all filed)

me-cli leg (`me-cli-csid-warning-surface`), device warning
(`device-csid-mismatch-warning`), Go JSON corpus ingestion
(`go-mk-vector-corpus-ingestion`), mk silent-correction reporting
(`mk-decode-silent-correction-reporting`), seat auto-partition
(`seat-merged-group-auto-partition`), md-codec, `--strict`/refusal mode,
the mk-vs-md `ChunkSetIdMismatch` naming collision.
