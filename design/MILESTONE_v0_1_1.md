# mk-codec v0.1.1 milestone scope

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans`. Per-phase Opus reviews dispatched per the established workflow; reports persisted to `design/agent-reports/v0-1-1-phase-<P>-review-<commit>.md`.

**Goal:** Ship `mk-codec v0.1.1` — a patch release that clears the v0.1-nice-to-have backlog and lands the test-corpus pre-BIP-submission items, without touching the wire format. v0.1.1 = "pure polish + corpus expansion." Doc-audit items (NUMS structural audit, HRP collision check, BIP cross-reference completeness) ship in a separate review-grade milestone (`design/MILESTONE_v0_1_pre_bip_audit.md`, TBD), not here.

**Wire-format invariant:** v0.1.1 emits and accepts byte-identical wire output to v0.1.0. The vector corpus expands; existing vectors round-trip unchanged; new vectors are additive. Patch-version semver discipline.

**Branch:** `feature/v0.1.1-implementation` from `mk-codec-v0.1.0` tag. Land all phases on this branch; tag `mk-codec-v0.1.1` after the final commit + CHANGELOG bump.

**Scope summary:**

| FOLLOWUPS id | Tier | Phase | Resolution |
|---|---|---|---|
| `cross-chunk-hash-test-fixture-stability` | v0.1-nice-to-have | 1 | code change to test-only file |
| `pipeline-decode-mixed-header-error-naming` | v0.1-nice-to-have | 1 | new `Error` variant + plumbing |
| `encode-with-chunk-set-id-singlestring-silent-ignore` | v0.1-nice-to-have | 1 | close as `wont-fix — moot per SPEC §2.4` |
| `vector-corpus-dictionary-coverage` | pre-bip-submission | 2 | add 9 clean vectors; bump corpus SHA pin |
| `decoder-error-variant-parity` | pre-bip-submission | 3 | schema-extend the corpus to support negative vectors |

**Out of scope (deferred to a separate milestone):**

- `nums-structural-audit`, `hrp-mk-collision-check`, `bip-cross-reference-completeness` — these are review-grade analytical artifacts. Each produces a standalone document plus a SPEC/BIP cross-reference pass; they're better delivered as a single pre-BIP-submission audit milestone with a single opus reviewer pass over the whole package, not as patch-release add-ons.
- All `cross-repo` items — depend on `descriptor-mnemonic` actions; mk1 v0.1.1 cannot land them.

**Spec references:**

- v0.1 spec: [`design/SPEC_mk_v0_1.md`](./SPEC_mk_v0_1.md) (frozen for v0.1.x).
- DECISIONS log: [`design/DECISIONS.md`](./DECISIONS.md) (D-1..D-15 closed).
- FOLLOWUPS source-of-truth: [`design/FOLLOWUPS.md`](./FOLLOWUPS.md).
- v0.1.0 implementation plan: [`design/IMPLEMENTATION_PLAN_mk_v0_1.md`](./IMPLEMENTATION_PLAN_mk_v0_1.md) (the format-shape reference for what the corpus expansions need to mirror).

---

## File structure

| File | Responsibility | Phase |
|---|---|---|
| `crates/mk-codec/src/string_layer/pipeline.rs` | Refine `decode_rejects_perturbed_cross_chunk_hash` test fixture | 1 |
| `crates/mk-codec/src/error.rs` | Add `Error::MixedHeaderTypes` variant | 1 |
| `crates/mk-codec/src/string_layer/pipeline.rs` | Replace `ChunkedHeaderMalformed` with `MixedHeaderTypes` at the SingleString-with-extras rejection site | 1 |
| `design/FOLLOWUPS.md` | Close `encode-with-chunk-set-id-singlestring-silent-ignore` as `wont-fix` with cross-link to SPEC §2.4 | 1 |
| `crates/mk-codec/src/bin/gen_mk_vectors.rs` | Add 9 fixtures (V9..V17) covering missing dictionary entries | 2 |
| `crates/mk-codec/tests/vectors/v0.1.json` | Regenerated, expanded from 8 to 17 vectors; new SHA-256 pin | 2 |
| `crates/mk-codec/tests/vectors.rs` | Update `V0_1_SHA256` constant | 2 |
| `crates/mk-codec/tests/vectors/v0.1.json` | Schema bump to v2 with `expected_error` field on negative vectors | 3 |
| `crates/mk-codec/tests/vectors.rs` | Schema-version-aware harness that dispatches clean vs negative vectors | 3 |
| `crates/mk-codec/Cargo.toml` | Version 0.1.0 → 0.1.1 | 4 |
| `CHANGELOG.md` | `[0.1.1]` section | 4 |
| `design/FOLLOWUPS.md` | Update item statuses to `resolved <COMMIT>` | 4 |

---

## Phase 0 — Branch + workspace prep

**Goal:** Set up the v0.1.1 branch off the v0.1.0 line.

### Task 0.1 — Branch

The v0.1.0 line currently has one post-tag commit on `feature/v0.1.0-implementation` — `21efbea docs: add CLAUDE.md for next-session context loading`, which is repo-meta documentation that should persist into v0.1.1 (CLAUDE.md is auto-loaded by Claude Code sessions; dropping it would degrade the worktree's AI-collaboration affordances). Two viable approaches:

- **(a) Branch from `feature/v0.1.0-implementation` head** (post-CLAUDE.md commit). Pro: zero cherry-picking. Con: branches from `21efbea` rather than the released tag, so the v0.1.1 branch's first divergence point is one commit ahead of the release tag.
- **(b) Branch from `mk-codec-v0.1.0` tag**, then cherry-pick `21efbea`. Pro: clean tag-aligned history. Con: requires a cherry-pick step.

**Recommended: (a).** The v0.1.0 release artifact is the tag itself; the v0.1.1 branch only needs to be wire-compatible with v0.1.0, and a `git diff mk-codec-v0.1.0..feature/v0.1.1-implementation` gives a clean patch view at any time. Choose (b) only if the v0.1.0 line acquires further post-tag commits that should NOT carry into v0.1.1.

- [ ] `git checkout -b feature/v0.1.1-implementation feature/v0.1.0-implementation` (approach (a)) **or** `git checkout -b feature/v0.1.1-implementation mk-codec-v0.1.0 && git cherry-pick 21efbea` (approach (b)). All phase commits land on the new branch.

### Task 0.2 — Toolchain sanity

- [ ] Confirm `cargo build`, `cargo test -p mk-codec`, `cargo test -p mk-codec --features gen-vectors`, `cargo clippy -p mk-codec --features gen-vectors --all-targets -- -D warnings`, and `cargo fmt --all -- --check` all pass on the v0.1.0 baseline.

---

## Phase 1 — v0.1-nice-to-have backlog clearance

**Goal:** Resolve the 3 items deferred from Phase 5/6 fixup commits during v0.1.0.

**Per-phase agent report:** `design/agent-reports/v0-1-1-phase-1-review-<commit>.md`.

### Task 1.1 — Robust cross-chunk-hash perturbation test

The current `decode_rejects_perturbed_cross_chunk_hash` test (`crates/mk-codec/src/string_layer/pipeline.rs::272`) perturbs the last byte of the last chunk's fragment, then re-encodes. The perturbation lands in 5-bit-symbol space after re-encoding, so BCH t=4 correction can in principle silently un-flip it back to a valid codeword whose recomputed hash happens to match. The fixture is currently fine, but a future fixture change could mask the test.

- [ ] **Step 1.1.1: Tests first** — write a new test that perturbs at the 5-bit-symbol layer *after* `encode_5bit_to_string` finishes and *before* the BCH checksum is appended. Specifically: compute the canonical chunked encoding, find the chunk_index of the last chunk, decode it back to (header_symbols, payload_symbols), modify **at least 5 payload symbols** that lie inside the cross-chunk-hash region (BCH `t = 4` covers up to 4 substitutions exactly; a 5+-symbol perturbation always exceeds the correction radius and the decoder must return either `Err(CrossChunkHashMismatch)` — if BCH fails-soft to a wrong-but-valid codeword whose recomputed hash mismatches — or `Err(BchUncorrectable)` — if BCH gives up). Re-encode with the same checksum function. Assert `decode(...)` returns one of `{Err(CrossChunkHashMismatch), Err(BchUncorrectable)}`; both indicate the perturbation was rejected, which is the property under test. The current test (`pipeline.rs:302-305`) accepts only `CrossChunkHashMismatch`, which is brittle under the same logic — relax the new test's accepting set to both variants.

- [ ] **Step 1.1.2: Replace the existing test** with the new one. Delete the brittle byte-flip variant.

- [ ] **Step 1.1.3: Verify** `cargo test -p mk-codec` passes and the new test name (`decode_rejects_perturbed_cross_chunk_hash_5_symbol_burst` or similar) reflects the BCH-distance discipline.

### Task 1.2 — `Error::MixedHeaderTypes` variant

Two call-sites in the current codebase reject header-type disagreement across a multi-string input list, both currently surfacing as `ChunkedHeaderMalformed("...")`:

- **Forward direction** (`crates/mk-codec/src/string_layer/pipeline.rs::137-139`): first string is `SingleString`, additional strings follow. Caught early in `pipeline::decode`.
- **Reverse direction** (`crates/mk-codec/src/string_layer/chunk.rs::170-174`): first chunk is `Chunked`, an internal chunk is `SingleString`. Caught inside `reassemble_from_chunks` after `pipeline::decode` falls through to the chunked branch.

A third site (`chunk.rs::123-127`) rejects "first chunk is `SingleString`" inside `reassemble_from_chunks`. This is unreachable from `pipeline::decode` (which intercepts the all-`SingleString` case earlier), but stays as defense-in-depth for any future direct caller of `reassemble_from_chunks`. **It keeps `ChunkedHeaderMalformed`** — the variant signals "this function expected all-Chunked input"; it is not a "header types disagree" condition.

- [ ] **Step 1.2.1: Tests first** — add **two** tests in `pipeline::tests`:
  - `decode_rejects_singlestring_then_chunked`: supplies `[SingleString_string, Chunked_string]`, asserts `Err(MixedHeaderTypes)`. Covers the forward direction.
  - `decode_rejects_chunked_then_singlestring`: supplies `[Chunked_string, SingleString_string]`, asserts `Err(MixedHeaderTypes)`. Covers the reverse direction.

- [ ] **Step 1.2.2: Add the variant** to `crates/mk-codec/src/error.rs` between `ChunkedHeaderMalformed` and `ChunkSetIdMismatch` (alphabetical-ish position; matches the existing string-layer-error grouping). Include rustdoc explaining when it fires vs `ChunkedHeaderMalformed`.

  ```rust
  /// Decoder received a multi-string input whose `SingleString` and
  /// `Chunked` header variants disagree across the supplied list.
  /// Either: first string is `SingleString` but additional strings
  /// follow (caught in `pipeline::decode`); or first chunk is
  /// `Chunked` but a later chunk is `SingleString` (caught in
  /// `reassemble_from_chunks`). Distinct from `ChunkedHeaderMalformed`
  /// (which covers issues *within* a declared-chunked set: bad
  /// chunk_index, bad total_chunks, etc.).
  #[error("mixed string-layer header types in input list")]
  MixedHeaderTypes,
  ```

- [ ] **Step 1.2.3: Plumbing** — migrate the two enumerated call-sites:
  - `pipeline.rs:137` `Error::ChunkedHeaderMalformed("multiple strings supplied with SingleString header")` → `Error::MixedHeaderTypes`.
  - `chunk.rs:171` `Error::ChunkedHeaderMalformed("single-string header mixed with chunked reassembly")` → `Error::MixedHeaderTypes`.
  - **DO NOT** migrate `chunk.rs:124` (the "first chunk SingleString" defense-in-depth check); it keeps `ChunkedHeaderMalformed("single-string header in multi-chunk reassembly")` as the API-contract violation signal.
  - Audit the four `decode_rejects_*` tests in `chunk.rs::tests` and `pipeline.rs::tests` to confirm none of them assert specifically on the migrated message text; if any do, update to `MixedHeaderTypes`.

- [ ] **Step 1.2.4: Verify** — backwards compatibility check: `Error` is `#[non_exhaustive]`, so existing exhaustive-match consumers won't break. The variant text differs, but downstream code that pattern-matches on variant identity rather than message text gets a more precise discriminator.

- [ ] **Step 1.2.5: Update `crate::error::tests::static_variants_render`** to include a case for the new variant (parameter-less — slots into the static-variant list).

### Task 1.3 — Close `encode-with-chunk-set-id-singlestring-silent-ignore` as wont-fix

The Phase 6 review observed that this item is moot in practice because SPEC §2.4 confirmed SingleString is unreachable for v0.1 conforming KeyCards. The "silent-ignore" branch in `encode_with_chunk_set_id` is dead code under the v0.1 wire format.

- [ ] **Step 1.3.1: Update FOLLOWUPS** — change the item's `Status:` from `open` to `wont-fix — moot per SPEC §2.4 (SingleString unreachable for v0.1 conforming KeyCards).`

- [ ] **Step 1.3.2: Add a sequencing requirement** to the FOLLOWUPS entry:

  > _"If a future format extension lands a smaller bytecode (e.g., Compact-65 per SPEC §3.6, which would drop xpub.version + xpub.parent_fingerprint and bring some bytecodes below 56 bytes), this item MUST be re-opened **before the format extension ships**. The silent-drop semantics is friendly today but masks an encoder-side determinism bug under any wire format that makes SingleString reachable. A pending-FOLLOWUPS-or-equivalent gate should be checked at any future smaller-bytecode design pass."_

### Task 1.4 — Build + commit + review

- [ ] **Step 1.4.1: Verify**

  ```bash
  cargo test -p mk-codec
  cargo test -p mk-codec --features gen-vectors
  cargo clippy -p mk-codec --features gen-vectors --all-targets -- -D warnings
  cargo fmt --all -- --check
  ```

- [ ] **Step 1.4.2: Commit**

  ```
  fix(mk-codec phase 1.1): clear v0.1-nice-to-have backlog
  
  - Adds Error::MixedHeaderTypes variant; pipeline::decode now
    surfaces this for header-type disagreement across the input
    string list (was overloaded onto ChunkedHeaderMalformed).
  - Hardens decode_rejects_perturbed_cross_chunk_hash test: perturbs
    at the 5-bit-symbol layer with a BCH-distance > 4 burst, removing
    the silent-un-flip risk.
  - Closes encode-with-chunk-set-id-singlestring-silent-ignore as
    wont-fix; SingleString is unreachable for v0.1 conforming KeyCards
    per SPEC §2.4.
  ```

- [ ] **Step 1.4.3: Phase 1 review**

  Dispatch Opus reviewer.

  - Files: `error.rs`, `pipeline.rs`, `FOLLOWUPS.md`.
  - Verify: new variant rustdoc adequate; test discipline holds; FOLLOWUPS closure rationale is sound.
  - Output: `design/agent-reports/v0-1-1-phase-1-review-<commit>.md`.

---

## Phase 2 — Vector corpus dictionary expansion

**Goal:** Add 9 vectors (V9..V17) covering the missing path-dictionary entries so a third-party encoder cannot pass the v0.1.1 corpus while having a bug in BIP 44/49/86 mainnet or testnet (other than 0x15) handling.

**Per-phase agent report:** `design/agent-reports/v0-1-1-phase-2-review-<commit>.md`.

### Missing dictionary entries (cross-checked against SPEC §3.5)

| Indicator | Path | Vector ID |
|---|---|---|
| 0x01 | `m/44'/0'/0'` (BIP 44 mainnet) | V9 |
| 0x02 | `m/49'/0'/0'` (BIP 49 mainnet) | V10 |
| 0x04 | `m/86'/0'/0'` (BIP 86 mainnet) | V11 |
| 0x06 | `m/48'/0'/0'/1'` (BIP 48 nested-segwit mainnet) | V12 |
| 0x11 | `m/44'/1'/0'` (BIP 44 testnet) | V13 |
| 0x12 | `m/49'/1'/0'` (BIP 49 testnet) | V14 |
| 0x13 | `m/84'/1'/0'` (BIP 84 testnet) | V15 |
| 0x14 | `m/86'/1'/0'` (BIP 86 testnet) | V16 |
| 0x17 | `m/87'/1'/0'` (BIP 87 testnet) | V17 |

(Indicator 0x16 = `m/48'/1'/0'/1'` testnet nested-segwit is intentionally skipped; it's tracked as `md-path-dictionary-0x16-gap` in FOLLOWUPS pending md1 dictionary update. mk1 cannot legitimately emit it until md1 closes the gap.)

### Task 2.1 — Add fixtures

- [ ] **Step 2.1.1**: Append 9 entries to `gen_mk_vectors.rs::fixtures()`. Each follows the existing pattern: distinct `seed_byte` (0x09..0x11), distinct `chunk_set_id` (memorable hex), correct `network` field, fingerprint state alternating between Some/None to extend coverage of bytecode-header-bit-2 paths beyond the existing V4/V7 split.

  Recommended split: V9, V10, V11, V13, V14, V15 with fingerprint present; V12, V16, V17 fingerprint omitted. Rationale: each new BIP/network pair is exercised under at least one fingerprint mode; cross-pair invariance is implicit (the bytecode-header-bit-2 path is independent of the path-dictionary indicator).

### Task 2.2 — Regenerate corpus + update SHA pin

- [ ] **Step 2.2.1**:

  ```bash
  cargo run --bin gen_mk_vectors --features gen-vectors
  sha256sum crates/mk-codec/tests/vectors/v0.1.json
  # paste the new hex into V0_1_SHA256 in tests/vectors.rs
  cargo test -p mk-codec --test vectors
  ```

- [ ] **Step 2.2.2**: Verify byte-determinism — re-run the generator into a tmp path, then `cmp -s` it against the on-disk file (catches stale generator state vs accidental re-write):

  ```bash
  cargo run --bin gen_mk_vectors --features gen-vectors -- --output /tmp/mk-vectors-regen.json
  cmp -s /tmp/mk-vectors-regen.json crates/mk-codec/tests/vectors/v0.1.json && echo "byte-deterministic ✓" || (echo "drift detected" && diff /tmp/mk-vectors-regen.json crates/mk-codec/tests/vectors/v0.1.json | head -20)
  ```

### Task 2.3 — Build + commit + review

- [ ] **Step 2.3.1: Verify** the full test suite passes (162 tests expected: 147 unit + 3 round_trip + 3 vectors + new tests from Phase 1; per-vector round-trip count grows from 8 to 17).

- [ ] **Step 2.3.2: Commit**

  ```
  feat(mk-codec phase 2): expand vector corpus to 17 entries
  
  Adds V9..V17 covering the 9 path-dictionary entries missing from
  v0.1.0's V1..V8 set:
    V9-V11  BIP 44 / 49 / 86 mainnet
    V12     BIP 48 nested-segwit mainnet
    V13-V17 testnet variants for BIP 44 / 49 / 84 / 86 / 87
  
  Regenerated tests/vectors/v0.1.json (SHA <new>); updated
  V0_1_SHA256 in tests/vectors.rs accordingly. Resolves
  vector-corpus-dictionary-coverage (FOLLOWUPS).
  ```

- [ ] **Step 2.3.3: Phase 2 review**

  Dispatch Opus reviewer.

  - Files: `gen_mk_vectors.rs`, `tests/vectors/v0.1.json` (spot-check 2-3 new vectors), `tests/vectors.rs`.
  - Verify: every new vector's `canonical_bytecode_hex` starts with the correct std-table indicator byte; networks match; SHA pin matches the on-disk file.
  - Output: `design/agent-reports/v0-1-1-phase-2-review-<commit>.md`.

---

## Phase 3 — Negative-vector corpus + schema bump

**Goal:** Add `expected_error` support to the vector schema so the corpus can pin one negative vector per `Error` variant (mapping SPEC §4 rules 1–14 to triggering inputs). Resolves `decoder-error-variant-parity`.

**Schema decision:** bump `schema` field from 1 to 2, adding a top-level `expected_error` field on every vector entry. Clean vectors emit `expected_error: null`; negative vectors set it to the rendered `Error` variant string. **The generator MUST always emit the field** (with `null` for clean vectors) — omitting it on clean vectors and emitting it on negative vectors would break byte-determinism, since `serde_json::Map`'s alphabetical key ordering produces a different SHA-256 depending on which keys are present. Bumping the schema field is informational; the harness gates per-vector on `expected_error.is_null()` vs `is_string()`.

**Per-phase agent report:** `design/agent-reports/v0-1-1-phase-3-review-<commit>.md`.

### Task 3.1 — Schema extension

- [ ] **Step 3.1.1**: Update `gen_mk_vectors.rs` to write `"schema": 2` and accept negative-fixture entries. Add a `FixtureKind` enum:

  ```rust
  enum FixtureKind {
      Clean,
      Negative {
          /// The pre-encoded mk1 string list to feed to the decoder.
          /// Bypasses the encoder so the fixture can construct
          /// inputs that no conforming encoder would emit.
          input_strings: Vec<String>,
          /// The `Error` variant the decoder MUST surface, identified
          /// by its `Display` string.
          expected_error: &'static str,
          /// One-line rationale of what the negative input exercises.
          why: &'static str,
      },
  }
  ```

  `FixtureSpec` gains a `kind: FixtureKind` field. Existing entries default to `Clean`.

### Task 3.2 — Negative vectors per Error variant

Add one negative vector per Error variant from `crates/mk-codec/src/error.rs`. Each requires a hand-constructed input that triggers the specific rejection. Vector IDs `N1`..`Nk` (where k = number of negative vectors).

Mandatory coverage (SPEC §4 rules + string-layer rules):

| Vector | Error variant | Trigger |
|---|---|---|
| N1 | `InvalidHrp` | string with HRP `bt1` |
| N2 | `MixedCase` | mk1 string with one ASCII-uppercase char in the data part |
| N3 | `InvalidStringLength` | string with 10-char data part (below regular-code minimum) |
| N4 | `InvalidChar` | string with `b` in the data part |
| N5 | `BchUncorrectable` | string with 5+ randomised substitutions |
| N6 | `UnsupportedCardType` | string-layer header with `type = 0x02` |
| N7 | `MalformedPayloadPadding` | SingleString-shaped string with non-zero pad bits in the final 5-bit symbol |
| N8 | `ChunkSetIdMismatch` | two-chunk input where chunk 1 has a different `chunk_set_id` from chunk 0 |
| N9 | `ChunkedHeaderMalformed` (chunk_index >= total_chunks) | second chunk declares `chunk_index = total_chunks` |
| N10 | `MixedHeaderTypes` (added in Phase 1) | `[SingleString, Chunked]` mixed input |
| N11 | `CrossChunkHashMismatch` | reassembled chunks whose trailing 4-byte hash doesn't match SHA-256 of bytecode |
| N12 | `UnsupportedVersion` | bytecode header with version=1 |
| N13 | `ReservedBitsSet` | bytecode header with bit 0, 1, or 3 set |
| N14 | `InvalidPolicyIdStubCount` | bytecode with stub_count=0 |
| N15 | `InvalidPathIndicator` | bytecode with path indicator 0x00 |
| N16 | `PathTooDeep` | explicit path with `count = 11` |
| N17 | `InvalidPathComponent` | explicit path with truncated LEB128 |
| N18 | `InvalidXpubVersion` | xpub with version 0xDEADBEEF |
| N19 | `InvalidXpubPublicKey` | xpub with non-curve-point bytes |
| N20 | `UnexpectedEnd` | bytecode truncated mid-xpub |
| N21 | `TrailingBytes` | bytecode with one extra byte after the xpub |
| N22 | `CardPayloadTooLarge` | 1693-byte hand-constructed bytecode |
| N23 | `ChunkedHeaderMalformed` (empty input) | empty `&[]` passed to `decode` (covers the second `ChunkedHeaderMalformed` call-site at `pipeline.rs:117-121`) |

(`FingerprintFlagMismatch` was retired during Phase 4; see SPEC §4 note.)

`ChunkedHeaderMalformed` is `String`-parameterized and is emitted from multiple distinct rejection conditions — the negative-vector parity contract therefore covers both call-site shapes (chunk-index OOB via N9; empty input via N23). The exhaustiveness gate in Step 3.3.2 only asserts variant-level coverage; per-call-site coverage is enforced by the table above and by Phase 1 plus Phase 2 unit tests in `chunk.rs::tests` and `pipeline.rs::tests`.

For each: write a `gen_negative_<id>` helper in `gen_mk_vectors.rs` that constructs the malformed input (often by manipulating the encoder pipeline at a specific layer to bypass validation). The helper returns a `(Vec<String>, &'static str)` pair (input_strings, expected_error rendered).

### Task 3.3 — Harness updates

- [ ] **Step 3.3.1**: Update `tests/vectors.rs::every_vector_round_trips` to dispatch on whether `expected_error` is null:

  - `null` (or absent): existing clean-vector logic runs.
  - present: call `decode(input_strings)`, assert it returns `Err(_)`, assert the rendered `Display` of the error matches `expected_error` byte-for-byte.

- [ ] **Step 3.3.2**: Add a separate test `every_error_variant_has_negative_vector` that asserts at least one negative vector exists per `Error` variant. Implementation: write the test as an in-crate exhaustive `match` over a constructed `Error` value of each variant (one arm per variant). `#[non_exhaustive]` blocks **external** exhaustive matching, but the test lives inside `mk-codec` so the compiler still warns when a new variant is added without an arm — equivalently strong as a `strum::EnumIter` derive without adding a dev-dependency. The match arms map each variant to its negative-vector ID(s) by string lookup against the corpus's `name` field, asserting the lookup succeeds. (md-codec uses `strum::EnumIter` for the same pattern; choosing in-crate `match` here keeps the dep tree minimal and gets the same compiler-enforcement.)

### Task 3.4 — Build + commit + review

- [ ] **Step 3.4.1: Verify**

  ```bash
  cargo run --bin gen_mk_vectors --features gen-vectors
  sha256sum crates/mk-codec/tests/vectors/v0.1.json
  # update V0_1_SHA256
  cargo test -p mk-codec
  ```

- [ ] **Step 3.4.2: Commit**

  ```
  feat(mk-codec phase 3): negative-vector corpus (decoder-error-variant-parity)
  
  Bumps vector schema 1 → 2 with optional `expected_error` field.
  Adds N1..N22 negative vectors covering every Error variant in
  crates/mk-codec/src/error.rs (SPEC §4 rules 1-14 + string-layer
  rules + structural variants). Each vector's input_strings is
  hand-constructed to trigger one specific rejection path; the
  harness asserts byte-equal `Display` rendering.
  
  Adds every_error_variant_has_negative_vector exhaustiveness test
  to gate the corpus invariant in CI.
  
  Resolves decoder-error-variant-parity (FOLLOWUPS).
  ```

- [ ] **Step 3.4.3: Phase 3 review**

  Dispatch Opus reviewer.

  - Files: `gen_mk_vectors.rs`, `tests/vectors.rs`, `tests/vectors/v0.1.json` (spot-check N1, N5, N7, N16, N22).
  - Verify: every Error variant has a triggering negative vector; rendered Display strings match across vector + harness; SHA pin matches.
  - Output: `design/agent-reports/v0-1-1-phase-3-review-<commit>.md`.

---

## Phase 4 — Release plumbing

**Goal:** Cargo bump 0.1.0 → 0.1.1; CHANGELOG; FOLLOWUPS status updates; tag.

### Task 4.1 — Cargo bump

- [ ] `crates/mk-codec/Cargo.toml`: `0.1.0` → `0.1.1`.

### Task 4.2 — CHANGELOG

- [ ] Append `[0.1.1]` section to `CHANGELOG.md`:

  ```markdown
  ## [0.1.1] — 2026-XX-XX
  
  Patch release: v0.1-nice-to-have backlog clearance + vector corpus expansion. Wire format unchanged from v0.1.0.
  
  ### Added
  - `Error::MixedHeaderTypes` variant for header-type disagreement across multi-string inputs (precise discriminator; previously surfaced as `ChunkedHeaderMalformed`).
  - 9 new clean vectors (V9..V17) covering all path-dictionary entries except the 0x16 testnet nested-segwit gap (cross-repo md1 dependency).
  - 22 negative vectors (N1..N22), one per `Error` variant, with `expected_error` schema field.
  - Vector-corpus schema bumped 1 → 2 to support negative vectors via `expected_error`. Schema-1 corpora remain readable by the harness.
  
  ### Changed
  - `decode_rejects_perturbed_cross_chunk_hash` test now perturbs at the 5-bit-symbol layer with a BCH-distance > 4 burst, removing the silent-un-flip risk in the v0.1.0 fixture.
  
  ### Resolved (FOLLOWUPS)
  - `cross-chunk-hash-test-fixture-stability`
  - `pipeline-decode-mixed-header-error-naming`
  - `vector-corpus-dictionary-coverage`
  - `decoder-error-variant-parity`
  - `encode-with-chunk-set-id-singlestring-silent-ignore` (closed as `wont-fix — moot per SPEC §2.4`)
  
  ### Notes
  - Wire format is byte-identical to v0.1.0; existing v0.1.0 strings round-trip unchanged.
  - Cross-implementations need to update their `V0_1_SHA256` pin to match the expanded corpus. The renamed `Error::MixedHeaderTypes` is exhaustive-match-safe because `Error` is `#[non_exhaustive]`.
  ```

### Task 4.3 — FOLLOWUPS status updates

- [ ] Update each resolved item's `Status:` line from `open` to `resolved <commit>`. Cross-link the commit SHA.

### Task 4.4 — Tag

- [ ] After Phase 4 commit lands and CI passes, request user authorisation before:

  ```bash
  git tag -a mk-codec-v0.1.1 -m "mk-codec v0.1.1 — backlog + corpus expansion"
  git push origin mk-codec-v0.1.1
  gh release create mk-codec-v0.1.1 --notes-from-tag  # or use the CHANGELOG body
  ```

### Task 4.5 — Final reconciliation

- [ ] **Step 4.5.1**: List all `design/agent-reports/v0-1-1-*` reports; verify every minor item is either resolved or recorded back into FOLLOWUPS at an appropriate tier.
- [ ] **Step 4.5.2**: Update project memory with the v0.1.1 ship state (parallel to `project_v0_1_0_shipped.md`).

---

## Test count expectations

| State | Unit | round_trip | vectors | Total |
|---|---|---|---|---|
| v0.1.0 (baseline) | 147 | 3 | 3 | 153 |
| After Phase 1 | 149 | 3 | 3 | 155 |
| After Phase 2 | 149 | 3 | 3 (parameterised over 17 vectors) | 155 |
| After Phase 3 | 149 | 3 | 4 (adds `every_error_variant_has_negative_vector`) | 156 |

Phase 1 adds **two** new unit tests (per Task 1.2.1: forward-direction `decode_rejects_singlestring_then_chunked` + reverse-direction `decode_rejects_chunked_then_singlestring`). Task 1.1 *replaces* `decode_rejects_perturbed_cross_chunk_hash` rather than adding a new test (count delta 0). Net: +2 unit tests after Phase 1.

Phase 2 doesn't add a new test name — `every_vector_round_trips` already iterates over the entire JSON, so adding 9 vectors expands coverage without growing the test count. Phase 3 adds one new exhaustiveness test (`every_error_variant_has_negative_vector`) and grows the iteration space inside `every_vector_round_trips` to ~40 entries (17 clean + 23 negative).

---

## Risks + tradeoffs

- **Schema bump in Phase 3 changes the corpus shape.** Cross-implementations validating against the v0.1.0 corpus need to migrate to schema 2 to consume v0.1.1. Mitigation: schema-1 readers still parse cleanly because the new `expected_error` field is additive (null on existing clean vectors); only consumers of negative vectors need to handle the new field.
- **`Error::MixedHeaderTypes` is observable behaviour change.** A v0.1.0 consumer that pattern-matches on `Error::ChunkedHeaderMalformed("multiple strings supplied with SingleString header")` (substring match on the message) will silently stop matching in v0.1.1. The variant is `#[non_exhaustive]`-safe but text-match-fragile; CHANGELOG calls this out.
- **Phase 3 negative-vector authoring depends on post-Phase-1 `Error::Display` strings.** The N10 vector (and any future negative vectors that map to `MixedHeaderTypes`) pins `expected_error: "mixed string-layer header types in input list"`. If Phase 3 vector authoring runs against a worktree that hasn't merged Phase 1, the rendered `Error::Display` reads `"chunked-header malformed: multiple strings supplied with SingleString header"` instead, and the negative vector ships with stale text. Mitigation: Phase 3 is sequenced strictly after Phase 1; the generator binary always reads `Error::Display` at run time from the current build, so re-running the generator post-Phase-1 produces correct strings. Any third-party validator that pinned `expected_error` against a pre-Phase-1 internal corpus build (none exists today, but this is a future hazard) would observe drift.
- **Negative-vector inputs are sensitive to encoder details.** A future encoder change that alters output byte ordering could invalidate negative vectors that were constructed by mutating clean-vector outputs. Phase 3 step 3.2 helpers should construct negatives bottom-up (from raw bytecode) rather than mutating encoder output where possible.

---

## Out of scope (re-stated for clarity)

- Wire-format changes — patch release semver discipline.
- NUMS structural audit, HRP collision check, BIP cross-reference completeness — these are review-grade analytical milestones, deferred to a separate `pre-bip-submission audit` milestone (TBD; would land before formal BIP submission and likely after md1's `chunk-set-id-rename` cross-repo work clears).
- All `cross-repo` items — depend on `descriptor-mnemonic` actions; v0.1.1 is mk1-only.
- crates.io publication — still deferred until pre-BIP-submission audit gates clear.

---

(End of mk-codec v0.1.1 milestone scope.)
